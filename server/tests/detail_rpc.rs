//! Wave 25 gate: DownloadChapter enqueues; the queue worker drives the At-Home flow to
//! completion (downloaded + page count recorded); ListChapters returns state as data.

use koma_server::db;
use koma_server::download::{enqueue, run_queue_once, RealRunner};
use koma_server::library::{LibraryDb, LibraryService};
use koma_server::pb as pb_types;
use koma_server::source::mangadex::MangaDexAdapter;
use serde_json::json;
use tonic::Request;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PAGE_BYTES: &[u8] = b"\xFF\xD8\xFF\xE0page-bytes";

async fn mock_athome() -> MockServer {
    let server = MockServer::start().await;
    let base = server.uri();
    Mock::given(method("GET"))
        .and(path("/at-home/server/ch-ext-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": "ok",
            "baseUrl": base,
            "chapter": { "hash": "h", "data": ["p0.img", "p1.img", "p2.img", "p3.img"] },
            "reportUrl": format!("{base}/report")
        })))
        .mount(&server)
        .await;
    for n in 0..4 {
        Mock::given(method("GET"))
            .and(path(format!("/data/h/p{n}.img").as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(PAGE_BYTES.to_vec()))
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(path("/report"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "result": "ok" })))
        .mount(&server)
        .await;
    server
}

async fn seeded(base: &str) -> (koma_server::DbPool, LibraryDb, std::path::PathBuf) {
    let root = db::temp_test_dir("wave25");
    std::fs::create_dir_all(&root).expect("root");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    koma_server::source::seed_builtin_sources(&pool).await.expect("builtin sources");
    sqlx::query("INSERT INTO manga (id, source_id, source_key, title) VALUES (1, 1, 'm-ext', 'M')")
        .execute(&pool)
        .await
        .expect("manga");
    sqlx::query(
        "INSERT INTO chapter (id, manga_id, source_key, number, storage_dir) VALUES (1, 1, 'ch-ext-1', '1', NULL)",
    )
    .execute(&pool)
    .await
    .expect("chapter");
    let lib = LibraryDb {
        pool: pool.clone(),
        mangadex: MangaDexAdapter::with_client(koma_server::fetch::test_client(), base),
    };
    (pool, lib, root)
}

#[tokio::test]
async fn download_chapter_enqueues_and_worker_completes_it() {
    let mock = mock_athome().await;
    let (pool, lib, root) = seeded(&mock.uri()).await;

    // The RPC only enqueues (the background worker in main drives the pass).
    lib.download_chapter(Request::new(pb_types::DownloadChapterRequest { chapter_id: 1 }))
        .await
        .expect("enqueue rpc");

    let (state,): (String,) =
        sqlx::query_as("SELECT state FROM chapter WHERE id = 1").fetch_one(&pool).await.expect("state");
    assert_eq!(state, "queued", "ListChapters must show the queue state immediately");

    // The worker pass (as main.rs's spawned loop runs it).
    let runner = RealRunner {
        http: koma_server::fetch::test_client(),
        adapter: MangaDexAdapter::with_client(koma_server::fetch::test_client(), &mock.uri()),
        pool: pool.clone(),
        storage_root: root.clone(),
    };
    let (ok, failed) = run_queue_once(&pool, &runner).await.expect("worker pass");
    assert_eq!((ok, failed), (1, 0), "one download driven to completion");

    let (state, pages, storage): (String, i64, Option<String>) =
        sqlx::query_as("SELECT state, pages, storage_dir FROM chapter WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("chapter row");
    assert_eq!(state, "downloaded");
    assert_eq!(pages, 4, "page count recorded by the worker");
    assert!(storage.is_some());

    // ListChapters returns the state as data.
    let chapters = lib
        .list_chapters(Request::new(pb_types::ListChaptersRequest { manga_id: 1 }))
        .await
        .expect("list")
        .into_inner();
    assert_eq!(chapters.items.len(), 1);
    assert_eq!(chapters.items[0].state, "downloaded");
    assert_eq!(chapters.items[0].pages, 4);
}

#[tokio::test]
async fn list_chapters_orders_latest_first() {
    let mock = MockServer::start().await;
    let (pool, lib, _root) = seeded(&mock.uri()).await;
    for (id, num) in [(2, "2"), (3, "3")] {
        sqlx::query("INSERT INTO chapter (id, manga_id, source_key, number) VALUES (?, 1, ?, ?)")
            .bind(id)
            .bind(format!("ch-ext-{id}"))
            .bind(num)
            .execute(&pool)
            .await
            .expect("extra chapter");
    }
    let chapters = lib
        .list_chapters(Request::new(pb_types::ListChaptersRequest { manga_id: 1 }))
        .await
        .expect("list")
        .into_inner();
    assert_eq!(chapters.items.len(), 3);
    assert_eq!(chapters.items[0].id, 3, "latest-added first");
    assert_eq!(chapters.items[2].id, 1);
}
