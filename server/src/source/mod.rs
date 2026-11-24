//! Source adapters (tier A/B policy): each adapter is a thin mapping from external API shapes
//! into Koma's own types, registered as source table rows (behavior in data).

pub mod mangadex;

/// Tier-A sources the binary ships with register themselves as rows (behavior in data:
/// the adapter is compiled in, so its registry row must exist on any fresh install —
/// found by the wave-24 gate: add-after-search failed on a fresh DB).
pub async fn seed_builtin_sources(pool: &crate::db::DbPool) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO source (key, name) VALUES ('mangadex', 'MangaDex')")
        .execute(pool)
        .await?;
    Ok(())
}
