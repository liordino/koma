//! Koma server library entry point.

pub mod auth;
pub mod db;
pub mod log;

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
impl pb::koma_server::Koma for KomaTransport {
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
/// consequence chain).
pub fn build_router(pool: DbPool, allowed_origins: &[&str]) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .map(|o| o.parse::<HeaderValue>().expect("valid origin"))
                .collect::<Vec<_>>(),
        )
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let grpc_router = tonic::service::Routes::new(pb::koma_server::KomaServer::new(KomaTransport))
        .into_axum_router();

    Router::new()
        .merge(grpc_router)
        .layer(middleware::from_fn_with_state(pool.clone(), auth::require_bearer))
        .layer(tonic_web::GrpcWebLayer::new())
        .layer(cors)

}

use tower_http::cors::CorsLayer;
