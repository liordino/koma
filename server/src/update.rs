//! Auto-update checker (M2 wave 16): library rows are upserted from their source by external
//! id. Idempotency contract: (manga_id, source_key) is UNIQUE on chapter, so re-running the
//! pass adds nothing new; `changes()` counts real inserts. Manual trigger in v1 (RPC).

use crate::db::DbPool;
use crate::fetch::FetchError;

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("fetch failed: {0}")]
    Fetch(#[from] FetchError),
    #[error("db failed: {0}")]
    Db(#[from] sqlx::Error),
}

/// The one thing an update pass needs from a source: ordered chapters by external manga id.
pub trait UpdatableSource {
    fn list_chapters(
        &self,
        manga_external_id: &str,
        limit: u32,
    ) -> impl std::future::Future<Output = Result<Vec<ChapterInfo>, FetchError>> + Send;
}

/// Adapted chapter facts needed for upsert (external id + chapter number).
#[derive(Debug, Clone)]
pub struct ChapterInfo {
    pub external_id: String,
    pub number: String,
}

/// One idempotent pass over every library manga of `source_key`.
/// Returns the number of NEW chapter rows inserted.
pub async fn run_update_pass<S: UpdatableSource>(
    pool: &DbPool,
    source: &S,
    source_key: &str,
    limit: u32,
) -> Result<i32, UpdateError> {
    let manga_rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT m.id, m.source_key FROM manga m JOIN source s ON s.id = m.source_id WHERE s.key = ?",
    )
    .bind(source_key)
    .fetch_all(pool)
    .await?;

    let mut added = 0i32;
    for (manga_id, manga_external_id) in manga_rows {
        let chapters = source.list_chapters(&manga_external_id, limit).await?;
        for chapter in chapters {
            // (manga_id, source_key) UNIQUE — INSERT OR IGNORE + changes() = true novelty count.
            let res = sqlx::query(
                "INSERT OR IGNORE INTO chapter (manga_id, source_key, number) VALUES (?, ?, ?)",
            )
            .bind(manga_id)
            .bind(&chapter.external_id)
            .bind(&chapter.number)
            .execute(pool)
            .await?;
            if res.rows_affected() > 0 {
                added += 1;
            }
        }
    }
    Ok(added)
}

impl UpdatableSource for crate::source::mangadex::MangaDexAdapter {
    async fn list_chapters(
        &self,
        manga_external_id: &str,
        limit: u32,
    ) -> Result<Vec<ChapterInfo>, FetchError> {
        Ok(self
            .chapters(manga_external_id, limit)
            .await?
            .into_iter()
            .map(|c| ChapterInfo { external_id: c.external_id, number: c.chapter })
            .collect())
    }}
