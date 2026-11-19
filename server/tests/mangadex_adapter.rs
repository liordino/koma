//! Wave 10 gate: MangaDex adapter maps recorded API shapes into Koma types, over the
//! SSRF-guarded OutboundClient (allowlist must include the mock's host for these tests).

use koma_server::fetch::OutboundClient;
use koma_server::source::mangadex::MangaDexAdapter;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn adapter(server: &MockServer) -> MangaDexAdapter {
    let http = OutboundClient::new(
        vec!["127.0.0.1".to_string()],
        1000,
        1000,
        std::time::Duration::from_secs(1),
    )
    .expect("client");
    MangaDexAdapter::with_client(
        std::sync::Arc::new(http),
        server.uri().as_str(),
    )
}

/// Recorded MangaDex search body shape: result/data[] with type=manga + cover_art relationship.
fn search_body() -> serde_json::Value {
    json!({
        "result": "ok",
        "data": [
            {
                "type": "manga",
                "id": "a1f939be-6e51-49ec-9394-c9939c63a302",
                "attributes": { "title": { "en": "One Piece" }, "description": { "en": "Roger D. Luffy!" } },
                "relationships": [
                    { "type": "cover_art", "attributes": { "fileName": "abc.jpg" } }
                ]
            },
            { "type": "manga", "id": "x9", "attributes": { "title": { "ja-ro": "Berserk" } } }
        ]
    })
}

fn feed_body() -> serde_json::Value {
    json!({
        "result": "ok",
        "data": [
            { "type": "chapter", "id": "ch2",
              "attributes": { "chapter": "1189", "title": "The End", "pages": 17, "scanlator": null } },
            { "type": "chapter", "id": "ch1",
              "attributes": { "chapter": "1188", "title": null, "pages": 21, "scanlator": "nolitoy" } }
        ]
    })
}


#[tokio::test]
async fn search_maps_titles_ids_and_covers() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/manga"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_body()))
        .mount(&server)
        .await;

    let results = adapter(&server).search("one piece", 20).await.expect("search");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].source_key, "mangadex");
    assert_eq!(results[0].external_id, "a1f939be-6e51-49ec-9394-c9939c63a302");
    assert_eq!(results[0].title, "One Piece");
    assert_eq!(results[0].cover_url.as_deref(), Some("https://mangadex.org/covers/abc.jpg"));
    // ja-ro fallback for titles without en:
    assert_eq!(results[1].title, "Berserk");
    assert!(results[1].cover_url.is_none());
}

#[tokio::test]
async fn detail_map_and_missing_data_is_an_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/manga/one-piece-id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": "ok",
            "data": [ { "type": "manga", "id": "one-piece-id",
                        "attributes": { "title": { "en": "One Piece" },
                                        "description": { "en": "Yohoho" } } } ]
        })))
        .mount(&server)
        .await;

    let d = adapter(&server).detail("one-piece-id").await.expect("detail");
    assert_eq!(d.title, "One Piece");
    assert_eq!(d.description.as_deref(), Some("Yohoho"));

    Mock::given(method("GET"))
        .and(path("/manga/empty"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "result": "ok", "data": [] })))
        .mount(&server)
        .await;
    let empty = adapter(&server).detail("empty").await;
    assert!(empty.is_err(), "missing data must be an error, not a panic");
}

#[tokio::test]
async fn chapter_list_maps_and_orders_as_api_sends() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/manga/one-piece-id/feed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(feed_body()))
        .mount(&server)
        .await;

    let chapters = adapter(&server).chapters("one-piece-id", 50).await.expect("feed");
    assert_eq!(chapters.len(), 2);
    assert_eq!(chapters[0].external_id, "ch2");
    assert_eq!(chapters[0].chapter, "1189");
    assert_eq!(chapters[0].pages, 17);
    assert_eq!(chapters[1].scanlator.as_deref(), Some("nolitoy"));
    assert!(chapters[1].title.is_none());
}
