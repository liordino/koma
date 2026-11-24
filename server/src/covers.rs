//! Cover art serving (M4 wave 23): auth-gated like /pages, consumed fetch-to-blob by the SPA
//! (decision 24/26 apply to cover bytes exactly as to page bytes).
//!
//! Cache: first request fetches the upstream URL through the outbound client (API plane,
//! allowlisted domains), stores the bytes under {storage}/covers/{id}.jpg, and every later
//! request serves from disk — one upstream fetch per cover, ever.

use std::path::PathBuf;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

use crate::db::DbPool;
use crate::fetch::SharedClient;

#[derive(Clone)]
pub struct CoversState {
    pub pool: DbPool,
    pub outbound: SharedClient,
    pub storage_root: PathBuf,
}

pub async fn get_cover(state: State<CoversState>, Path(manga_id): Path<i64>) -> Response {
    let CoversState { pool, outbound, storage_root } = state.0;

    // No expect/unwrap on the request path: every failure maps to a status.
    let row = sqlx::query_as::<_, (Option<String>,)>("SELECT cover_url FROM manga WHERE id = ?")
        .bind(manga_id)
        .fetch_optional(&pool)
        .await;
    let cover_url = match row {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Ok(Some((url,))) => match url {
            Some(u) if !u.is_empty() => u,
            _ => return StatusCode::NOT_FOUND.into_response(),
        },
    };

    let cache_dir = storage_root.join("covers");
    let cache_file = cache_dir.join(format!("{manga_id}.jpg"));

    if let Ok(bytes) = tokio::fs::read(&cache_file).await {
        return (StatusCode::OK, [(header::CONTENT_TYPE, "image/jpeg")], bytes).into_response();
    }

    // Disk miss: fetch upstream once through the guarded client.
    match outbound.get_bytes(&cover_url, crate::fetch::Plane::Api).await {
        Ok(bytes) => {
            let _ = tokio::fs::create_dir_all(&cache_dir).await;
            let _ = tokio::fs::write(&cache_file, &bytes).await;
            (StatusCode::OK, [(header::CONTENT_TYPE, "image/jpeg")], bytes).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
