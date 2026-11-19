//! Wave 3 gate: bearer-token auth + unary gRPC-Web transport is real (decision 24).

use axum::http::{header, Request, StatusCode};
use tower::ServiceExt;

use koma_server::db;

use std::sync::atomic::AtomicU32;
static SERIAL: AtomicU32 = AtomicU32::new(0);

async fn seed_pool() -> koma_server::DbPool {
    let test_dir = db::temp_test_dir("wave3");
    std::fs::create_dir_all(&test_dir).expect("test dir");
    let url = format!(
        "sqlite://{}?mode=rwc",
        test_dir.join("koma.db").display()
    );
    let _ = SERIAL;
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    koma_server::auth::seed_owner(&pool, "test-token")
        .await
        .expect("seed");
    pool
}

/// gRPC-Web unary frame: 1 flag byte (0 = uncompressed) + 4-byte BE length + message bytes.
fn grpc_web_frame(message: &[u8]) -> Vec<u8> {
    let mut frame = vec![0u8];
    frame.extend_from_slice(&(message.len() as u32).to_be_bytes());
    frame.extend_from_slice(message);
    frame
}

fn ping_request(token: Option<&str>) -> Request<axum::body::Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/koma.v1.KomaService/Ping")
        .header("content-type", "application/grpc-web+proto");
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    builder
        .body(axum::body::Body::from(grpc_web_frame(&[])))
        .unwrap()
}

#[tokio::test]
async fn rpc_without_token_is_401() {
    let pool = seed_pool().await;
    let router = koma_server::build_router(pool, &["http://localhost:5173"], std::env::temp_dir());
    let res = router.clone().oneshot(ping_request(None)).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "no token must be 401"
    );
}

#[tokio::test]
async fn rpc_with_valid_token_answers_unary_grpc_web() {
    let pool = seed_pool().await;
    let router = koma_server::build_router(pool, &["http://localhost:5173"], std::env::temp_dir());
    let res = router
        .clone()
        .oneshot(ping_request(Some("test-token")))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "valid token must pass");
    let ct = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(
        ct.starts_with("application/grpc"),
        "gRPC-Web response content type, got {ct}"
    );
    // unary: exactly one data frame (pong is 22 bytes: flag+len+message "pong"? len covers payload)
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(!body.is_empty(), "unary response must carry a data frame");
    assert_eq!(body[0], 0, "uncompressed frame flag");
}

#[tokio::test]
async fn wrong_token_is_401() {
    let pool = seed_pool().await;
    let router = koma_server::build_router(pool, &["http://localhost:5173"], std::env::temp_dir());
    let res = router
        .oneshot(ping_request(Some("wrong-token")))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn cors_preflight_from_allowed_origin_passes() {
    let pool = seed_pool().await;
    let router = koma_server::build_router(pool, &["http://localhost:5173"], std::env::temp_dir());
    let req = Request::builder()
        .method("OPTIONS")
        .uri("/koma.v1.KomaService/Ping")
        .header("origin", "http://localhost:5173")
        .header("access-control-request-method", "POST")
        .body(axum::body::Body::from(Vec::new()))
        .unwrap();
    let res = router.oneshot(req).await.unwrap();
    assert!(
        res.status().is_success() || res.status() == StatusCode::METHOD_NOT_ALLOWED,
        "preflight, got {}",
        res.status()
    );
    assert!(
        res.headers().get("access-control-allow-origin").is_some(),
        "allowed origin must be echoed"
    );
}
