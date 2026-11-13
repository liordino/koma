//! Koma server library entry point.

pub mod auth;
pub mod db;
pub mod log;
pub mod pages;

pub use db::DbPool;

pub mod pb {
    tonic::include_proto!("koma.v1");
}



use axum::http::{header, HeaderValue, Method};
use axum::{middleware, Router};

/// Transport-spike implementation of koma.v1.Koma.
#[derive(Default)]
pub struct KomaTransport;

#[tonic::async_trait]
impl pb::koma_service_server::KomaService for KomaTransport {
    async fn ping(
        &self,
        _request: tonic::Request<pb::PingRequest>,
    ) -> Result<tonic::Response<pb::PingResponse>, tonic::Status> {
        Ok(tonic::Response::new(pb::PingResponse {
            message: "pong".into(),
        }))
    }
}

/// Build the HTTP app: unary gRPC-Web transport (tonic + tonic-web), bearer-token middleware on
/// every route (decision 24), CORS restricted to the configured UI origins (separate-origins
/// consequence chain), and path-safe binary page serving under /pages (decision 26).
pub fn build_router(
    pool: DbPool,
    allowed_origins: &[&str],
    storage_root: std::path::PathBuf,
) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .map(|o| o.parse::<HeaderValue>().expect("valid origin"))
                .collect::<Vec<_>>(),
        )
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    // GrpcWeb wraps only the gRPC routes: it must not sit over /pages, where plain binary
    // GETs would be rejected by the gRPC-Web content-type negotiation (found by the probe).
    let grpc_router = tonic::service::Routes::new(
        pb::koma_service_server::KomaServiceServer::new(KomaTransport),
    )
    .into_axum_router()
    .layer(tonic_web::GrpcWebLayer::new());

    let pages_state = pages::PagesState {
        pool: pool.clone(),
        base_dir: storage_root,
    };

    Router::new()
        .merge(grpc_router)
        .route(
            "/pages/{chapter_id}/{page_n}",
            axum::routing::get(pages::get_page).with_state(pages_state),
        )
        .layer(middleware::from_fn_with_state(pool.clone(), auth::require_bearer))
        .layer(cors)

}

use tower_http::cors::CorsLayer;
