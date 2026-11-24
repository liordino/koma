//! Wave 23 gate: the /covers route fetches upstream ONCE through the guarded client,
//! then serves from the disk cache; missing covers 404 cleanly (SPA placeholder path).

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use koma_server::db;

const COVER_BYTES: &[u8] = b"\xFF\xD8\xFF\xE0cover-art-bytes";

async fn seeded(cover_url: Option<String>) -> (koma_server::DbPool, std::path::PathBuf) {
    let root = db::temp_test_dir("wave23-covers");
    std::fs::create_dir_all(&root).expect("root");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    koma_server::auth::seed_owner(&pool, "t").await.expect("seed");
    sqlx::query("INSERT INTO source (id, key, name) VALUES (1, 'mangadex', 'MangaDex')")
        .execute(&pool)
        .await
        .expect("source");
    sqlx::query("INSERT INTO manga (id, source_id, source_key, title, cover_url) VALUES (1, 1, 'm-ext', 'M', ?)")
        .bind(cover_url)
        .execute(&pool)
        .await
        .expect("manga with cover");
    (pool, root)
}

fn get(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header(header::AUTHORIZATION, "Bearer t")
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn cover_fetches_once_then_serves_from_disk_cache() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/cover.jpg"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(COVER_BYTES.to_vec()))
        .mount(&upstream)
        .await;

    let (pool, root) = seeded(Some(format!("{}/cover.jpg", upstream.uri()))).await;
    let router = koma_server::build_router(
        pool.clone(),
        &["http://localhost:5173"],
        root.clone(),
        koma_server::fetch::test_client(),
    );

    // First request: upstream fetch + cache write.
    let res = router.clone().oneshot(get("/covers/1")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), COVER_BYTES);

    // Second request: served from disk — upstream still saw exactly ONE request.
    let res = router.clone().oneshot(get("/covers/1")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), COVER_BYTES);

    let hits = upstream.received_requests().await.expect("request log").len();
    assert_eq!(hits, 1, "disk cache must serve the second request");

    // Cache file on disk proves the persistence half of the contract.
    assert!(root.join("covers").join("1.jpg").exists());
}

#[tokio::test]
async fn missing_covers_404_for_placeholder_fallback() {
    // No upstream involved: the row has no cover_url, so the route must 404 immediately.
    let (pool, root) = seeded(None).await;
    let router = koma_server::build_router(
        pool,
        &["http://localhost:5173"],
        root,
        koma_server::fetch::test_client(),
    );

    let res = router.clone().oneshot(get("/covers/1")).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND, "no cover_url -> 404");

    let res = router.oneshot(get("/covers/999")).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND, "unknown manga -> 404");
}
