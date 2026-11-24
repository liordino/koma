//! Core-loop benchmark, device half (wave 14): find → add → download a chapter, timed.
//! Run ON the pinned device; uses the real MangaDex API through this network.
//! Usage: KOMA_TOKEN=t ./koma-bench-loop [search query]

use koma_server::db;
use koma_server::library::{LibraryDb, LibraryService};
use koma_server::pb as pb_types;
use koma_server::source::mangadex::MangaDexAdapter;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = std::env::args().nth(1).unwrap_or_else(|| "one piece".into());
    let db_url = std::env::var("KOMA_DB_URL").unwrap_or_else(|_| "sqlite://koma.db?mode=rwc".into());
    let storage_root = std::env::var("KOMA_STORAGE").unwrap_or_else(|_| "./storage".into());
    let token = std::env::var("KOMA_TOKEN").unwrap_or_else(|_| "koma-dev-token".into());

    let pool = db::connect(&db_url).await?;
    db::migrate(&pool).await?;
    koma_server::auth::seed_owner(&pool, &token).await?;
    let lib = LibraryDb { pool: pool.clone() };

    // The source registry is data (behavior in data): the benchmark registers its source.
    sqlx::query("INSERT OR IGNORE INTO source (key, name) VALUES ('mangadex', 'MangaDex')")
        .execute(&pool)
        .await?;

    let adapter = MangaDexAdapter::production()?;

    // --- find ---
    let t0 = std::time::Instant::now();
    let hits = adapter.search(&query, 5).await.map_err(|e| format!("search: {e}"))?;
    let find_ms = t0.elapsed().as_millis();
    let first = hits.first().ok_or("no search results")?.clone();
    println!("{{\"event\":\"find\",\"ms\":{find_ms},\"title\":{:?}}}", first.title);

    // --- add (into category 0; category membership is orthogonal to the loop) ---
    let t1 = std::time::Instant::now();
    let summary = lib
        .add_manga(Request::new(pb_types::AddMangaRequest {
            source_key: first.source_key.clone(),
            external_id: first.external_id.clone(),
            title: first.title.clone(),
            cover_url: first.cover_url.clone().unwrap_or_default(),
            description: first.description.clone().unwrap_or_default(),
            category_id: 0,
        }))
        .await
        .map_err(|e| format!("add: {e}"))?
        .into_inner()
        .manga
        .unwrap_or_default();
    let add_ms = t1.elapsed().as_millis();
    println!("{{\"event\":\"add\",\"ms\":{add_ms},\"id\":{}}}", summary.id);

    // --- download first chapter (At-Home, real network) ---
    let t2 = std::time::Instant::now();
    let chapters = adapter
        .chapters(&first.external_id, 10)
        .await
        .map_err(|e| format!("chapters: {e}"))?;
    let chapter = chapters.first().ok_or("no chapters")?.clone();
    let ath = adapter
        .at_home(&chapter.external_id)
        .await
        .map_err(|e| format!("at-home: {e}"))?;

    sqlx::query(
        "INSERT OR IGNORE INTO chapter (manga_id, source_key, number, storage_dir) VALUES (?, ?, ?, NULL)",
    )
    .bind(summary.id)
    .bind(&chapter.external_id)
    .bind(&chapter.chapter)
    .execute(&pool)
    .await
    .map_err(|e| format!("chapter row: {e}"))?;
    let (row_id,): (i64,) = sqlx::query_as("SELECT id FROM chapter WHERE source_key = ?")
        .bind(&chapter.external_id)
        .fetch_one(&pool)
        .await?;

    let n = koma_server::download::download_chapter(
        &MangaDexAdapter::shared_production_client(),
        &adapter,
        &koma_server::download::DownloadContext {
            pool: &pool,
            storage_root: std::path::Path::new(&storage_root),
            chapter_row_id: row_id,
            chapter_external_id: &chapter.external_id,
            source_key: "mangadex",
            report: ath.report.as_ref().map(|(u, b)| (u.as_str(), b)),
        },
    )
    .await
    .map_err(|e| format!("download: {e}"))?;
    let dl_ms = t2.elapsed().as_millis();
    println!("{{\"event\":\"download\",\"ms\":{dl_ms},\"pages\":{n},\"chapter\":{:?}}}", chapter.chapter);

    let total = find_ms + add_ms + dl_ms;
    println!("{{\"event\":\"loop_device_half\",\"ms\":{total}}}");
    Ok(())
}
