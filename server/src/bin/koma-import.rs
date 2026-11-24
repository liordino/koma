//! koma-import — staged .tachibk import CLI (v1's import tooling; the RPC is future UI).
//! Usage: KOMA_DB_URL=... koma-import <backup.tachibk>
//! Stages per decision 14: decode -> VACUUM INTO snapshot -> single-transaction upserts.

use koma_server::db;
use koma_server::import_tachibk::{decode, import};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: koma-import <backup.tachibk>")?;
    let bytes = std::fs::read(&path)?;
    let backup = decode(&bytes)?;

    let db_url = std::env::var("KOMA_DB_URL")
        .unwrap_or_else(|_| "sqlite://koma.db?mode=rwc".to_string());
    let pool = db::connect(&db_url).await?;
    db::migrate(&pool).await?;

    let backup_dir = koma_server::backup_dir();
    let report = import(&pool, &backup, &backup_dir).await?;
    println!(
        "imported {} manga, {} chapters, {} categories, {} memberships (skipped {} non-favorites); snapshot: {:?}",
        report.manga_imported,
        report.chapters_imported,
        report.categories_imported,
        report.memberships_imported,
        report.skipped_non_favorite,
        report.snapshot_path
    );
    Ok(())
}
