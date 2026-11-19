//! Chapter download worker — single-writer discipline (decision 8), path-safe storage layout,
//! At-Home image-plane pulls.
//!
//! Chapter bytes live under `{storage_root}/{source_key}/chapters/{external_id}/page_{n}.jpg`.
//! The storage_dir column carries only the RELATIVE part (path-safe page serving re-checks the
//! canonical prefix at serve time; defense in depth here is cheap and symmetric).

use std::path::Path;

use serde_json::Value;

use crate::db::DbPool;
use crate::fetch::{FetchError, Plane, SharedClient};

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("fetch failed: {0}")]
    Fetch(#[from] FetchError),
    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("db failed: {0}")]
    Db(#[from] sqlx::Error),
    #[error("page {0} exceeds size cap")]
    PageTooLarge(usize),
}

/// The download worker needs one thing from a source adapter: the ordered list of image URLs
/// plus the (optional) report-key duty. Trait keeps the worker adapter-agnostic (behavior in
/// data: any source row can be backed by an implementation of this).
pub trait PageSource {
    /// Ordered page URLs + the report request to send after a successful download.
    fn at_home_urls_and_report(
        &self,
        chapter_external_id: &str,
    ) -> impl std::future::Future<Output = AtHomeUrls> + Send;
}

/// Size cap per page file — mirrors the page-serving cap (defense in depth at write time too).
pub const MAX_PAGE_BYTES: usize = crate::pages::MAX_PAGE_BYTES as usize;

/// Return type of PageSource::at_home_urls_and_report.
pub type AtHomeUrls = Result<(Vec<String>, Option<(String, Value)>), DownloadError>;

/// Everything a chapter download needs beyond the page source itself.
pub struct DownloadContext<'a> {
    pub pool: &'a DbPool,
    pub storage_root: &'a Path,
    pub chapter_row_id: i64,
    pub chapter_external_id: &'a str,
    pub source_key: &'a str,
    /// The At-Home report keyed by URL; sent after a successful download.
    pub report: Option<(&'a str, &'a Value)>,
}

/// Relative storage directory for a chapter; no traversal characters are produced here
/// because `source_key` and `external_id` are registry keys (validated at registration).
pub fn storage_dir_for(source_key: &str, chapter_external_id: &str) -> String {
    // Belt: assert keys are path-safe by construction; a hostile key fails fast here.
    for part in [source_key, chapter_external_id] {
        assert!(
            !part.contains(['/', '\\', '.', '\0'])
                && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "unsafe source/chapter key: {part}"
        );
    }
    format!("{source_key}/chapters/{chapter_external_id}")
}

/// Download one chapter: pull pages on the At-Home plane, write them, update the DB row,
/// then fire the report. Returns the number of pages stored.
pub async fn download_chapter<S: PageSource>(
    http: &SharedClient,
    source: &S,
    ctx: &DownloadContext<'_>,
) -> Result<usize, DownloadError> {
    let (urls, _from_adapter_report) = source
        .at_home_urls_and_report(ctx.chapter_external_id)
        .await?;

    let rel_dir = storage_dir_for(ctx.source_key, ctx.chapter_external_id);
    let abs_dir = ctx.storage_root.join(&rel_dir);
    std::fs::create_dir_all(&abs_dir)?;

    for (idx, url) in urls.iter().enumerate() {
        let bytes = http.get_bytes(url, Plane::AtHome).await?;
        if bytes.len() > MAX_PAGE_BYTES {
            return Err(DownloadError::PageTooLarge(bytes.len()));
        }
        tokio::fs::write(abs_dir.join(format!("page_{idx}.jpg")), &bytes).await?;
    }

    sqlx::query("UPDATE chapter SET downloaded = 1, storage_dir = ? WHERE id = ?")
        .bind(&rel_dir)
        .bind(ctx.chapter_row_id)
        .execute(ctx.pool)
        .await?;

    if let Some((url, body)) = ctx.report {
        // Report failure must not fail the download (data duty vs API hygiene).
        let _ = http.post_json(url, Plane::Api, body).await;
    }

    Ok(urls.len())
}

/// Convenience for tests/tools: verify a downloaded chapter is resolvable by /pages' resolver.
pub fn verify_pages_resolvable(
    storage_root: &Path,
    rel_dir: &str,
    pages: usize,
) -> Result<(), std::io::Error> {
    for n in 0..pages {
        let p = std::path::Path::new(storage_root).join(rel_dir).join(format!("page_{n}.jpg"));
        if !p.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("missing page {n}"),
            ));
        }
    }
    Ok(())
}

// ---- Download queue (M2 wave 15) ---------------------------------------------------------------
//
// DB-backed FIFO: `queued_at` orders the work, `state` is the queue state as data. The worker
// is a supervised loop: one task at a time, a failed task is contained (state='failed' +
// error text) and the worker moves to the next item — a failure never poisons the queue.



#[derive(Debug, Clone, sqlx::FromRow)]
pub struct QueueTask {
    pub chapter_row_id: i64,
    pub external_id: String,
    pub source_key: String,
}

/// Enqueue a chapter for download (idempotent: re-enqueue clears failure state, does not
/// reset an already-downloaded chapter).
pub async fn enqueue(pool: &DbPool, chapter_row_id: i64) -> Result<(), DownloadError> {
    sqlx::query(
        "UPDATE chapter SET state = 'queued', state_error = NULL, \
         queued_at = COALESCE(queued_at, strftime('%s','now')) WHERE id = ? AND state != 'downloaded'",
    )
    .bind(chapter_row_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Pop the oldest queued row (FIFO by queue time, ties by row id).
pub async fn next_queued(pool: &DbPool) -> Result<Option<QueueTask>, DownloadError> {
    let row: Option<QueueTask> = sqlx::query_as(
        "SELECT c.id AS chapter_row_id, c.source_key AS external_id, s.key AS source_key FROM chapter c          JOIN manga m ON m.id = c.manga_id          JOIN source s ON s.id = m.source_id          WHERE c.state = 'queued'          ORDER BY c.queued_at, c.id LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(DownloadError::Db)?;
    Ok(row)
}

/// The worker's unit of work — injectable so the queue logic is testable deterministically.
pub trait QueueRunner: Send + Sync {
    fn run(
        &self,
        task: &QueueTask,
    ) -> impl std::future::Future<Output = Result<usize, DownloadError>> + Send;
}

/// One supervised pass over the queue: FIFO, failure-contained, stops when drained.
/// Returns (processed_ok, failed).
pub async fn run_queue_once<R: QueueRunner>(
    pool: &DbPool,
    runner: &R,
) -> Result<(usize, usize), DownloadError> {
    let mut ok = 0;
    let mut failed = 0;
    loop {
        let Some(task) = next_queued(pool).await? else { break };
        sqlx::query("UPDATE chapter SET state = 'downloading' WHERE id = ?")
            .bind(task.chapter_row_id)
            .execute(pool)
            .await?;
        match runner.run(&task).await {
            Ok(_) => {
                sqlx::query("UPDATE chapter SET state = 'downloaded', state_error = NULL WHERE id = ?")
                    .bind(task.chapter_row_id)
                    .execute(pool)
                    .await?;
                ok += 1;
            }
            Err(e) => {
                sqlx::query("UPDATE chapter SET state = 'failed', state_error = ? WHERE id = ?")
                    .bind(e.to_string())
                    .bind(task.chapter_row_id)
                    .execute(pool)
                    .await?;
                failed += 1;
            }
        }
    }
    Ok((ok, failed))
}

/// The production runner: resolves the adapter for the task's source row, fetches the At-Home
/// block, downloads, and fires the report. Single-writer discipline: exactly one runner per app.
pub struct RealRunner {
    pub http: SharedClient,
    pub pool: DbPool,
    pub storage_root: std::path::PathBuf,
}

impl QueueRunner for RealRunner {
    async fn run(&self, task: &QueueTask) -> Result<usize, DownloadError> {
        // Source resolution is data: the source row's key selects the adapter.
        let source_key = task.source_key.as_str();
        if source_key != "mangadex" {
            return Err(DownloadError::Db(sqlx::Error::RowNotFound)); // unknown adapter → failed, contained
        }
        let adapter = crate::source::mangadex::MangaDexAdapter::production()?;
        let ath = adapter
            .at_home(&task.external_id)
            .await
            .map_err(DownloadError::Fetch)?;
        let ctx = DownloadContext {
            pool: &self.pool,
            storage_root: &self.storage_root,
            chapter_row_id: task.chapter_row_id,
            chapter_external_id: &task.external_id,
            source_key: &task.source_key,
            report: ath.report.as_ref().map(|(u, b)| (u.as_str(), b)),
        };
        download_chapter(&self.http, &adapter, &ctx).await
    }
}
