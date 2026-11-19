//! Wave 9 gate: outbound-fetch module is real — SSRF rejection happens BEFORE any wire
//! request, allowlisted requests parse, and the dual-plane rate limiter is measurable.

use koma_server::fetch::{OutboundClient, Plane};
use serde_json::Value;
use std::time::{Duration, Instant};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(allowlist: &[&str], api: usize, home: usize, window: Duration) -> OutboundClient {
    OutboundClient::new(allowlist.iter().map(|s| s.to_string()).collect(), api, home, window)
        .expect("client")
}

#[tokio::test]
async fn allowlisted_mock_serves_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/ping"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
        .mount(&server)
        .await;

    let c = client(&["127.0.0.1"], 1000, 1000, Duration::from_secs(1));
    let v: Value = c
        .get_json(&format!("{}/api/ping", server.uri()), Plane::Api)
        .await
        .expect("allowlisted request must pass");
    assert_eq!(v["ok"], serde_json::json!(true));
}

#[tokio::test]
async fn disallowed_host_hits_zero_requests_on_the_server() {
    // Mock stands up and would answer anything — the client must never dial it.
    let server = MockServer::start().await;

    let c = client(&["mangadex.org"], 1000, 1000, Duration::from_secs(1));
    let err = c
        .get_json::<Value>(&format!("{}/api/ping", server.uri()), Plane::Api)
        .await
        .expect_err("host is not allowlisted");
    assert!(err.to_string().contains("allowlist"), "got {err}");

    let hits = server
        .received_requests()
        .await
        .expect("received request log")
        .len();
    assert_eq!(hits, 0, "SSRF guard must reject pre-connection; wire saw {hits}");
}

#[tokio::test]
async fn lookalike_domain_is_rejected() {
    // evil-mangadex.org must NOT count as mangadex.org.
    let c = client(&["mangadex.org"], 1000, 1000, Duration::from_secs(1));
    let err = c
        .get_json::<Value>("http://evil-mangadex.org/api", Plane::Api)
        .await
        .expect_err("lookalike must be rejected");
    assert!(err.to_string().contains("allowlist"), "got {err}");
}

#[tokio::test]
async fn at_home_plane_enforcement_is_visible_in_timing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"image-bytes".to_vec()))
        .mount(&server)
        .await;

    // At-Home plane budget 1/window(400ms): 3 pulls must take >= 400ms.
    let c = client(&["127.0.0.1"], 1000, 1, Duration::from_millis(400));
    let t0 = Instant::now();
    for _ in 0..3u8 {
        let bytes = c
            .get_bytes(&format!("{}/img/0.jpg", server.uri()), Plane::AtHome)
            .await
            .expect("allowlisted image pull");
        assert_eq!(&bytes[..5], b"image");
    }
    let elapsed = t0.elapsed();
    assert!(
        elapsed >= Duration::from_millis(380),
        "image-plane budget must throttle, took {elapsed:?}"
    );
}
