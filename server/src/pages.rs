//! Path-safe page image serving over plain HTTP raw binary (decision 26: fetch-to-blob auth —
//! the bearer middleware on the router already gates this route).
//!
//! Contract: `/pages/{chapter_id}/{page_n}` → DB lookup for the chapter's `storage_dir`, bounded
//! `page_n`, canonicalize + prefix check against the storage root, then a bounded read.
//! The traversal regression test from day one guards the LLM-codegen bug class named in BRIEF.md.

use std::path::{Component, PathBuf};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
pub const MAX_PAGE_N: u32 = 2_000;
pub const MAX_PAGE_BYTES: i64 = 64 * 1024 * 1024;

#[derive(Clone)]
pub struct PagesState {
    pub pool: crate::db::DbPool,
    /// Storage root; chapter `storage_dir` values are relative to this and only this.
    pub base_dir: PathBuf,
}

/// Reject `..` components early — belt to the canonicalize braces.
fn storage_dir_is_safe(storage_dir: &str) -> bool {
    use std::path::Path;
    let p = Path::new(storage_dir);
    !storage_dir.starts_with(['/', '\\'])
        && PathBuf::from(storage_dir)
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
        && !p.is_absolute()
}

/// Resolve and validate the file path for one page. Returns the canonical absolute path or None.
pub fn resolve_page_path(
    base_dir: &std::path::Path,
    storage_dir: &str,
    page_n: u32,
) -> Option<PathBuf> {
    if !storage_dir_is_safe(storage_dir) {
        return None;
    }
    let path = base_dir.join(storage_dir).join(format!("page_{page_n}.jpg"));
    let canonical = path.canonicalize().ok()?;
    let canonical_root = base_dir.canonicalize().ok()?;
    if canonical.starts_with(&canonical_root) {
        Some(canonical)
    } else {
        None
    }
}

pub async fn get_page(
    state: State<PagesState>,
    Path((chapter_id, page_n)): Path<(i64, u32)>,
) -> Response {
    let PagesState { pool, base_dir } = state.0;

    if page_n > MAX_PAGE_N {
        return (StatusCode::BAD_REQUEST, "page_n out of bounds").into_response();
    }

    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT storage_dir FROM chapter WHERE id = ?")
            .bind(chapter_id)
            .fetch_optional(&pool)
            .await
            .expect("chapter lookup");
    let Some((Some(storage_dir),)) = row else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Some(path) = resolve_page_path(&base_dir, &storage_dir, page_n) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let bytes = match tokio::fs::read(&path).await {
        Ok(b) => b,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    let len = bytes.len() as i64;
    if len > MAX_PAGE_BYTES {
        return (StatusCode::PAYLOAD_TOO_LARGE, "page too large").into_response();
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/jpeg")],
        bytes,
    )
        .into_response()
}
