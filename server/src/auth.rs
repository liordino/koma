//! Bearer-token middleware (decision 24): every API route, including `/pages` (fetch-to-blob),
//! requires a token. Tokens are stored hashed; the presented token is hashed for comparison.

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};
use sqlx::Sqlite;

use crate::db::DbPool;

/// Hex sha-256 of a presented token — the storage format for `user.token_hash`.
pub fn hash_token(token: &str) -> String {
    let mut h = Sha256::new();
    h.update(token.as_bytes());
    hex::encode(h.finalize())
}

/// Extract and validate the `Authorization: Bearer <token>` header against the users table.
pub async fn require_bearer(
    State(pool): State<DbPool>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let presented = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let presented_hash = hash_token(presented);
    let known: Option<(i64,)> = sqlx::query_as("SELECT id FROM user WHERE token_hash = ?")
        .bind(&presented_hash)
        .fetch_one(&pool)
        .await
        .ok();

    if known.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}

/// Seed the single-operator account if absent (v1 is single-user; multi-user is post-v1).
pub async fn seed_owner(pool: &sqlx::Pool<Sqlite>, token: &str) -> Result<(), sqlx::Error> {
    let hash = hash_token(token);
    sqlx::query("INSERT OR IGNORE INTO user (name, token_hash) VALUES ('owner', ?)")
        .bind(hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub fn unauthorized() -> Response {
    StatusCode::UNAUTHORIZED.into_response()
}
