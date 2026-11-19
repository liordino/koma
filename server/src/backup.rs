//! Delete-after-read safety chain (M2 wave 18). Stop conditions from BRIEF.md, verbatim:
//! "Cleanup blocked without a fresh backup; permanent purge as an explicit, per-category,
//! user-confirmed action — not a seeded default on the primary category."
//!
//! Freshness: a backup is fresh for a category when its timestamp is >= the newest
//! trash_moved_at among that category's chapters.

use std::path::Path;

use serde::{Deserialize, Serialize};
use sqlx::{Column, Row};

use crate::db::DbPool;

#[derive(Debug, thiserror::Error)]
pub enum SafetyError {
    #[error("category not found")]
    NotFound,
    #[error("purge refused: confirmation flag missing (explicit user action required)")]
    Unconfirmed,
    #[error("purge refused: backup is stale — export a fresh metadata backup first (newest trash move: {0})")]
    StaleBackup(i64),
    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("db failed: {0}")]
    Db(#[from] sqlx::Error),
    #[error("serialization failed: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize)]
struct BackupDoc {
    exported_at: i64,
    sources: Vec<serde_json::Value>,
    #[serde(rename = "mangas")]
    manga: Vec<serde_json::Value>,
    chapters: Vec<serde_json::Value>,
    categories: Vec<serde_json::Value>,
    memberships: Vec<serde_json::Value>,
}

async fn dump_rows(pool: &DbPool, sql: &str) -> Result<Vec<serde_json::Value>, SafetyError> {
    let rows = sqlx::query(sql).fetch_all(pool).await?;
    let doc_rows = rows
        .into_iter()
        .map(|r| {
            let mut map = serde_json::Map::new();
            for (i, c) in r.columns().iter().enumerate() {
                let v: Option<serde_json::Value> = if let Ok(x) = r.try_get::<i64, _>(i) {
                    Some(serde_json::json!(x))
                } else if let Ok(x) = r.try_get::<Option<i64>, _>(i) {
                    x.map(|x| serde_json::json!(x))
                } else if let Ok(x) = r.try_get::<String, _>(i) {
                    Some(serde_json::json!(x))
                } else if let Ok(x) = r.try_get::<Option<String>, _>(i) {
                    x.map(|x| serde_json::json!(x))
                } else if let Ok(x) = r.try_get::<f64, _>(i) {
                    Some(serde_json::json!(x))
                } else {
                    None
                };
                map.insert(c.name().to_string(), v.unwrap_or(serde_json::Value::Null));
            }
            serde_json::Value::Object(map)
        })
        .collect();
    Ok(doc_rows)
}

/// Metadata-only JSON export. Returns (path, exported_at).
pub async fn export_metadata(pool: &DbPool, backup_dir: &Path) -> Result<(std::path::PathBuf, i64), SafetyError> {
    std::fs::create_dir_all(backup_dir)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let doc = BackupDoc {
        exported_at: now,
        sources: dump_rows(pool, "SELECT * FROM source").await?,
        manga: dump_rows(pool, "SELECT * FROM manga").await?,
        chapters: dump_rows(pool, "SELECT * FROM chapter").await?,
        categories: dump_rows(pool, "SELECT * FROM category").await?,
        memberships: dump_rows(pool, "SELECT * FROM manga_category").await?,
    };
    let path = backup_dir.join(format!("backup-{now}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;
    Ok((path, now))
}

/// The guarded purge. Refuses: unconfirmed, unknown category, or stale backup.
pub async fn purge_category(
    pool: &DbPool,
    trash_root: &Path,
    category_id: i64,
    confirm: bool,
) -> Result<i32, SafetyError> {
    if !confirm {
        return Err(SafetyError::Unconfirmed);
    }
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM category WHERE id = ?")
        .bind(category_id)
        .fetch_optional(pool)
        .await?;
    if exists.is_none() {
        return Err(SafetyError::NotFound);
    }

    let eligible: Vec<(i64, Option<i64>, Option<String>)> = sqlx::query_as(
        "SELECT c.id, c.trash_moved_at, c.trash_dir FROM chapter c \
         JOIN manga_category mc ON mc.manga_id = c.manga_id \
         WHERE mc.category_id = ? AND c.trash_dir IS NOT NULL",
    )
    .bind(category_id)
    .fetch_all(pool)
    .await?;

    let backup_at: Option<i64> = sqlx::query_as::<_, (i64,)>("SELECT last_backup_at FROM category WHERE id = ?")
        .bind(category_id)
        .fetch_optional(pool)
        .await?
        .map(|r| r.0);

    let newest_move = eligible.iter().filter_map(|(_, ts, _)| *ts).max().unwrap_or(0);
    if !eligible.is_empty() && backup_at.unwrap_or(0) < newest_move {
        return Err(SafetyError::StaleBackup(newest_move));
    }

    let mut purged = 0i32;
    for (chapter_id, _, trash_dir) in &eligible {
        if let Some(rel) = trash_dir {
            let dir = trash_root.join(rel);
            if dir.exists() {
                std::fs::remove_dir_all(dir)?;
            }
        }
        sqlx::query("DELETE FROM chapter WHERE id = ?")
            .bind(chapter_id)
            .execute(pool)
            .await?;
        purged += 1;
    }
    Ok(purged)
}
