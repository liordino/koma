//! Read tracking + trash window (M2 wave 17). The read predicate is an EXPLICIT mark (v1).
//! When a read-marked chapter belongs to a delete_after_read category, its files move to the
//! trash root and a retention timestamp is recorded — nothing is ever purged automatically
//! (decision 9 + BRIEF stop conditions: purge is explicit, per-category, user-confirmed).

use std::path::{Path, PathBuf};

use crate::db::DbPool;

#[derive(Debug, thiserror::Error)]
pub enum TrashError {
    #[error("chapter not found")]
    NotFound,
    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("db failed: {0}")]
    Db(#[from] sqlx::Error),
}

/// Mark a chapter read; if any owning category has delete_after_read=1, move its files into
/// the trash root and record the retention timestamp. Returns (moved, retention_until_epoch).
pub async fn mark_read(
    pool: &DbPool,
    chapter_id: i64,
    trash_root: &Path,
) -> Result<(bool, i64), TrashError> {
    let row: (String, Option<String>, String) = sqlx::query_as(
        "SELECT c.source_key, c.storage_dir, s.key FROM chapter c \
         JOIN manga m ON m.id = c.manga_id JOIN source s ON s.id = m.source_id \
         WHERE c.id = ?",
    )
    .bind(chapter_id)
    .fetch_optional(pool)
    .await?
    .ok_or(TrashError::NotFound)?;

    let is_delete_after_read: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM manga_category mc JOIN category cat ON cat.id = mc.category_id \
         WHERE mc.manga_id = (SELECT manga_id FROM chapter WHERE id = ?) AND cat.delete_after_read = 1",
    )
    .bind(chapter_id)
    .fetch_one(pool)
    .await?;
    let delete_after_read = is_delete_after_read.0 > 0;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let retention_until = now + 24 * 7 * 3600; // 7-day window, recorded — never auto-purged

    if delete_after_read {
        if let (Some(rel_dir), true) = (&row.1, !row.1.as_deref().unwrap_or_default().is_empty()) {
            let src_dir: PathBuf = rel_dir.to_owned().into();
            // Storage root lives next to the DB conventionally; resolve via storage env marker
            // recorded at download time: the caller passes the base through trash_root's parent.
            let storage_root = trash_root.parent().unwrap_or(Path::new("."));
            let from = storage_root.join(&src_dir);
            let to = trash_root.join(format!("ch{chapter_id}-{now}"));
            if from.exists() {
                std::fs::create_dir_all(&to)?;
                for entry in std::fs::read_dir(&from)? {
                    let entry = entry?;
                    let dest = to.join(entry.file_name());
                    std::fs::rename(entry.path(), dest)?;
                }
                std::fs::remove_dir(&from)?;
            }
            // reset the chapter row: files left the serving path; re-download stays possible
            sqlx::query(
                "UPDATE chapter SET downloaded = 0, storage_dir = NULL, state = 'pending', \
                 read_marked_at = ?, trash_moved_at = ?, trash_dir = ? WHERE id = ?",
            )
            .bind(now)
            .bind(now)
            .bind(
                to.strip_prefix(storage_root)
                    .unwrap_or(&to)
                    .to_string_lossy()
                    .to_string(),
            )
            .bind(chapter_id)
            .execute(pool)
            .await?;
            return Ok((true, retention_until));
        }
    }

    sqlx::query("UPDATE chapter SET read_marked_at = ? WHERE id = ?")
        .bind(now)
        .bind(chapter_id)
        .execute(pool)
        .await?;
    Ok((false, retention_until))
}

/// Chapters eligible for a purge decision (retention elapsed) — listing only; purge itself is
/// wave 18's explicit, per-category, user-confirmed action and is never automatic.
#[allow(dead_code)]
pub async fn retention_elapsed(pool: &DbPool, trash_root: &Path) -> Result<Vec<(i64, String)>, TrashError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let rows: Vec<(i64, Option<String>)> = sqlx::query_as(
        "SELECT id, trash_dir FROM chapter WHERE trash_dir IS NOT NULL AND trash_moved_at + 604800 <= ?",
    )
    .bind(now)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, dir)| (id, trash_root.join(dir.unwrap_or_default()).display().to_string()))
        .collect())
}
