//! Wave 11 gate: the At-Home flow + download worker produce a chapter whose pages the
//! /pages route resolves — the full image pipeline, mocked at the wire level.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{method, path as wpath};
use wiremock::{Mock, MockServer, ResponseTemplate};

use koma_server::fetch::OutboundClient;
use koma_server::source::mangadex::MangaDexAdapter;
use koma_server::{db, download, pages};

const PAGE_BYTES: &[u8] = b"\xFF\xD8\xFF\xE0-fake-jpeg-payload-";

fn client() -> std::sync::Arc<OutboundClient> {
    std::sync::Arc::new(
        OutboundClient::new(
            vec!["127.0.0.1".to_string()],
            1000,
            1000,
            std::time::Duration::from_secs(1),
        )
        .expect("client"),
    )
}

async fn seed_pool(root: &std::path::Path) -> koma_server::DbPool {
    let url = format!(
        "sqlite://{}?mode=rwc",
        root.join("koma.db").display()
    );
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    koma_server::auth::seed_owner(&pool, "t").await.expect("seed");
    pool
}

/// Mock: /at-home/server/ch-ext returns 4 files; /data/{hash}/{file} serves bytes;
/// /report accepts the report POST.
async fn mock_athome() -> MockServer {
    let server = MockServer::start().await;
    let base = server.uri();
    Mock::given(method("GET"))
        .and(wpath("/at-home/server/ch-ext-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": "ok",
            "baseUrl": base,
            "chapter": {
                "hash": "abc123",
                "data": ["p0.img", "p1.img", "p2.img", "p3.img"],
                "dataSaver": []
            },
            "reportUrl": format!("{base}/report")
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(wpath("/data/abc123/p0.img"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(PAGE_BYTES.to_vec()))
        .mount(&server)
        .await;
    for f in ["p1.img", "p2.img", "p3.img"] {
        Mock::given(method("GET"))
            .and(wpath(format!("/data/abc123/{f}").as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(PAGE_BYTES.to_vec()))
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(wpath("/report"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "result": "ok" })))
        .mount(&server)
        .await;
    server
}

fn adapter(server: &MockServer) -> MangaDexAdapter {
    MangaDexAdapter::with_client(client(), server.uri().as_str())
}

#[tokio::test]
async fn end_to_end_download_and_serve() {
    let server = mock_athome().await;
    let root = db::temp_test_dir("wave11-a");
    std::fs::create_dir_all(&root).expect("root");
    let pool = seed_pool(&root).await;

    sqlx::query("INSERT INTO source (id, key, name) VALUES (1, 'mangadex', 'MangaDex')")
        .execute(&pool)
        .await
        .expect("source");
    sqlx::query("INSERT INTO manga (id, source_id, source_key, title) VALUES (1, 1, 'bench', 'Bench')")
        .execute(&pool)
        .await
        .expect("manga");
    sqlx::query(
        "INSERT OR IGNORE INTO chapter (id, manga_id, source_key, number, storage_dir) VALUES (1, 1, 'bench-ch', '1187', NULL)",
    )
    .execute(&pool)
    .await
    .expect("chapter");

    let adapter = adapter(&server);
    let report_url = format!("{}/report", server.uri());
    let report_body = json!({ "status": 200, "data": { "empty": false } });
    let ctx = download::DownloadContext {
        pool: &pool,
        storage_root: &root,
        chapter_row_id: 1,
        chapter_external_id: "ch-ext-1",
        source_key: "mangadex",
        report: Some((report_url.as_str(), &report_body)),
    };

    let stored = download::download_chapter(&client(), &adapter, &ctx)
        .await
        .expect("download");
    assert_eq!(stored, 4);

    // Files on disk under the storage root, relative dir in the DB:
    assert!(download::verify_pages_resolvable(&root, "mangadex/chapters/ch-ext-1", 4).is_ok());
    let row: (i32, Option<String>) =
        sqlx::query_as("SELECT downloaded, storage_dir FROM chapter WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("row");
    assert_eq!(row.0, 1);
    assert_eq!(row.1.as_deref(), Some("mangadex/chapters/ch-ext-1"));

    // The /pages route serves the freshly stored bytes (auth + fetch-to-blob path):
    let router = koma_server::build_router(pool, &["http://localhost:5173"], root.clone(), koma_server::fetch::test_client());
    let req = Request::builder()
        .method("GET")
        .uri("/pages/1/0")
        .header(header::AUTHORIZATION, "Bearer t")
        .body(Body::empty())
        .unwrap();
    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // The report POST reached the mock with the required payload:
    let reqs = server
        .received_requests()
        .await
        .expect("request log");
    let report_seen = reqs
        .iter()
        .any(|r| r.url.path() == "/report" && !r.body.is_empty());
    assert!(report_seen, "report-key POST must be sent");
}

#[tokio::test]
async fn page_too_large_is_rejected_at_write_time() {
    let server = MockServer::start().await;
    let base = server.uri();
    // One huge page: exceeds the 64 MB cap set at read time; must be rejected at write.
    Mock::given(method("GET"))
        .and(wpath("/at-home/server/ch-big"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": "ok", "baseUrl": base,
            "chapter": { "hash": "h", "data": ["big.img"] },
            "reportUrl": null
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(wpath("/data/h/big.img"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(
            vec![0u8; (pages::MAX_PAGE_BYTES as usize) + 1],
        ))
        .mount(&server)
        .await;

    let root = db::temp_test_dir("wave11-b");
    std::fs::create_dir_all(&root).expect("root");
    let pool = seed_pool(&root).await;
    sqlx::query("INSERT INTO source (id, key, name) VALUES (1, 'mangadex', 'MangaDex')")
        .execute(&pool)
        .await
        .expect("source");
    sqlx::query("INSERT INTO manga (id, source_id, source_key, title) VALUES (2, 1, 'b', 'B')")
        .execute(&pool)
        .await
        .expect("manga");
    sqlx::query(
        "INSERT OR IGNORE INTO chapter (id, manga_id, source_key, number, storage_dir) VALUES (2, 2, 'bench-ch', '1', NULL)",
    )
    .execute(&pool)
    .await
    .expect("chapter");

    let adapter = adapter(&server);
    let ctx = download::DownloadContext {
        pool: &pool,
        storage_root: &root,
        chapter_row_id: 2,
        chapter_external_id: "ch-big",
        source_key: "mangadex",
        report: None,
    };
    let err = download::download_chapter(&client(), &adapter, &ctx).await;
    assert!(
        matches!(err, Err(download::DownloadError::PageTooLarge(_))),
        "oversized page must be rejected at write time, got {err:?}"
    );
}

#[tokio::test]
async fn unsafe_keys_fail_fast() {
    // Path-safety starts at key validation: traversal-shaped keys must panic loudly.
    let result = std::panic::catch_unwind(|| download::storage_dir_for("../evil", "ch-1"));
    assert!(result.is_err(), "traversal source key must be rejected");

    let result = std::panic::catch_unwind(|| download::storage_dir_for("mangadex", "a/b"));
    assert!(result.is_err(), "traversal chapter key must be rejected");

    // Legit keys produce a relative, traversal-free dir.
    assert_eq!(
        download::storage_dir_for("mangadex", "ch-ext-1"),
        "mangadex/chapters/ch-ext-1"
    );
}
