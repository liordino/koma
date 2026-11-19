//! Koma server library entry point.

pub mod auth;
pub mod db;
pub mod download;
pub mod library;
pub mod log;
pub mod trash;
pub mod update;
pub mod source;
pub mod fetch;
pub mod pages;

pub use db::DbPool;

/// Trash root: a SIBLING of the storage root so moved files leave /pages' canonicalized prefix
/// (the serve path), not another subdirectory of it.
pub fn trash_root() -> std::path::PathBuf {
    let storage = std::env::var("KOMA_STORAGE").unwrap_or_else(|_| "./storage".into());
    std::path::Path::new(&storage)
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("trash")
}

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
        // connect-es protocol headers must be allowed or every browser RPC dies on preflight.
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("connect-protocol-version"),
            header::HeaderName::from_static("connect-timeout-ms"),
            header::HeaderName::from_static("grpc-timeout"),
            header::HeaderName::from_static("x-user-agent"),
            header::HeaderName::from_static("x-grpc-web"),
        ]);

    // GrpcWeb wraps only the gRPC routes: it must not sit over /pages, where plain binary
    // GETs would be rejected by the gRPC-Web content-type negotiation (found by the probe).

    // Both tonic services share ONE Routes (one axum fallback) — merging two
    // into_axum_router() outputs panics (each carries its own fallback).
    let grpc_router = tonic::service::Routes::new(pb::koma_service_server::KomaServiceServer::new(KomaTransport))
        .add_service(pb::library_server::LibraryServer::new(library::LibraryDb {
            pool: pool.clone(),
        }))
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
