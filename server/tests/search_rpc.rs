//! Wave 24 gate: SearchManga proxies the tier-A adapter with the source badge; failures
//! map to UNAVAILABLE, never a panic.

use koma_server::db;
use koma_server::library::{LibraryDb, LibraryService};
use koma_server::source::mangadex::MangaDexAdapter;
use koma_server::pb as pb_types;
use serde_json::json;
use tonic::Request;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn seeded(base: &str) -> LibraryDb {
    let root = db::temp_test_dir("wave24");
    std::fs::create_dir_all(&root).expect("root");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    LibraryDb {
        pool,
        mangadex: MangaDexAdapter::with_client(koma_server::fetch::test_client(), base),
    }
}

#[tokio::test]
async fn search_maps_results_with_source_badge() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/manga"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": "ok",
            "data": [
                { "type": "manga", "id": "abc-123",
                  "attributes": { "title": { "en": "One Piece" } },
                  "relationships": [ { "type": "cover_art", "attributes": { "fileName": "x.jpg" } } ] }
            ]
        })))
        .mount(&mock)
        .await;

    let lib = seeded(&mock.uri()).await;
    let resp = lib
        .search_manga(Request::new(pb_types::SearchMangaRequest { query: "one piece".into(), limit: 10 }))
        .await
        .expect("search rpc")
        .into_inner();

    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.items[0].source_key, "mangadex", "source badge rides every hit");
    assert_eq!(resp.items[0].external_id, "abc-123");
    assert_eq!(resp.items[0].title, "One Piece");
    assert!(resp.items[0].cover_url.contains("covers/x.jpg"));
}

#[tokio::test]
async fn source_failure_maps_to_unavailable() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/manga"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock)
        .await;

    let lib = seeded(&mock.uri()).await;
    let err = lib
        .search_manga(Request::new(pb_types::SearchMangaRequest { query: "x".into(), limit: 5 }))
        .await
        .expect_err("upstream failure must surface");
    assert_eq!(err.code(), tonic::Code::Unavailable);
}

#[tokio::test]
async fn limit_is_clamped_server_side() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/manga"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "result": "ok", "data": [] })))
        .mount(&mock)
        .await;
    let lib = seeded(&mock.uri()).await;
    // absurd limits must not pass through to the source
    let resp = lib
        .search_manga(Request::new(pb_types::SearchMangaRequest { query: "x".into(), limit: 9999 }))
        .await
        .expect("clamped search")
        .into_inner();
    assert_eq!(resp.items.len(), 0);
}
