//! Staged `.tachibk` import (M2/M3 wave 19; decision 14).
//!
//! Stages: decode (gzip protobuf, public Apache-2.0 Mihon schema) → dry-run report →
//! `VACUUM INTO` snapshot (the rollback-safe point) → single-transaction commit with
//! external-ID upserts. Idempotency: UNIQUE keys + INSERT OR IGNORE; re-import adds nothing.
//!
//! Source registry is data (decision 24's posture): the backup's own `backupSources`
//! (name → id) registers rows; keys are ASCII slugs of the names. Only registered sources
//! can ever download (adapters are code); metadata-only sources import fine.

use std::path::{Path, PathBuf};

use prost::Message;

/// Mihon/Tachiyomi backup proto (subset — unknown fields ignored by prost).
pub mod mihon_pb {
    include!(concat!(env!("OUT_DIR"), "/mihon.backup.rs"));
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {

    #[error("protobuf decode failed: {0}")]
    Proto(#[from] prost::DecodeError),
    #[error("db failed: {0}")]
    Db(#[from] sqlx::Error),
    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Default, serde::Serialize)]
pub struct ImportReport {
    pub manga_seen: u32,
    pub manga_imported: u32,
    pub chapters_imported: u32,
    pub categories_imported: u32,
    pub memberships_imported: u32,
    /// source key → manga count in the backup (favorites only imported)
    pub by_source: std::collections::BTreeMap<String, u32>,
    /// manga skipped for being non-favorites (not in the library)
    pub skipped_non_favorite: u32,
    pub snapshot_path: Option<PathBuf>,
}

fn slug(name: &str) -> String {
    let slug: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if slug.is_empty() { "src-unknown".to_string() } else { slug }
}

fn slug_conflict_free(entries: &[(String, String)]) -> std::collections::BTreeMap<i64, String> {
    // source id → key; same-slug collisions get suffixes (MangaBats vs mangabats…)
    let mut map = std::collections::BTreeMap::new();
    let mut seen: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for (name, id) in entries {
        let base = slug(name);
        let n = seen.entry(base.clone()).or_insert(0);
        *n += 1;
        let key = if *n == 1 { base } else { format!("{base}-{n}") };
        map.insert(id.parse::<i64>().unwrap_or(0), key);
    }
    map
}

/// Read + decode a `.tachibk` (gzipped protobuf).
pub fn decode(input_bytes: &[u8]) -> Result<mihon_pb::Backup, ImportError> {
    let raw = gzip_decode(input_bytes)?;
    mihon_pb::Backup::decode(&raw[..]).map_err(ImportError::Proto)
}

fn gzip_decode(bytes: &[u8]) -> Result<Vec<u8>, ImportError> {
    use std::io::Read;
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(bytes).read_to_end(&mut out)?;
    Ok(out)
}

/// Dry-run: counts by source, no writes.
pub fn dry_run(backup: &mihon_pb::Backup) -> ImportReport {
    let mut report = ImportReport::default();
    let id_to_key = backup_by_source_map(backup);
    for m in &backup.backup_manga {
        // Mihon v2 backups include browsed/cached manga too; Koma imports all of them as
        // metadata and records which were actual library favorites (decision 14: mapping
        // verified against the owner's real export — favorites are NOT a filter here).
        let key = id_to_key
            .get(&m.source)
            .cloned()
            .unwrap_or_else(|| format!("src-{}", m.source));
        if m.favorite.unwrap_or(false) {
            *report.by_source.entry(key.clone()).or_insert(0) += 1;
        }
        report.manga_seen += 1;
    }
    report.categories_imported = backup.backup_categories.len() as u32;
    report
}

fn backup_by_source_map(backup: &mihon_pb::Backup) -> std::collections::BTreeMap<i64, String> {
    let entries: Vec<(String, String)> = backup
        .backup_sources
        .iter()
        .map(|s| (s.name.clone().unwrap_or_default(), s.source_id.to_string()))
        .collect();
    slug_conflict_free(&entries)
}

/// Staged import: `VACUUM INTO` snapshot, then a single-transaction commit.
pub async fn import(
    pool: &sqlx::SqlitePool,
    backup: &mihon_pb::Backup,
    storage_backup_dir: &Path,
) -> Result<ImportReport, ImportError> {
    let mut report = dry_run(backup);
    let id_to_key = backup_by_source_map(backup);

    // Stage: snapshot (VACUUM INTO) — the rollback point before any commit.
    std::fs::create_dir_all(storage_backup_dir)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let snapshot: PathBuf = storage_backup_dir.join(format!("pre-import-{now}-{nanos}.sqlite"));
    sqlx::query("VACUUM INTO ?")
        .bind(snapshot.display().to_string())
        .execute(pool)
        .await?;
    report.snapshot_path = Some(snapshot);

    let mut tx = pool.begin().await?;

    // Categories first (membership references them).
    for cat in &backup.backup_categories {
        sqlx::query("INSERT OR IGNORE INTO category (name) VALUES (?)")
            .bind(&cat.name)
            .execute(&mut *tx)
            .await?;
    }
    report.categories_imported = backup.backup_categories.len() as u32;

    for m in &backup.backup_manga {
        let key = id_to_key
            .get(&m.source)
            .cloned()
            .unwrap_or_else(|| format!("src-{}", m.source));
        sqlx::query("INSERT OR IGNORE INTO source (key, name) VALUES (?, ?)")
            .bind(&key)
            .bind(&key)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT OR IGNORE INTO manga (source_id, source_key, title)              SELECT id, ?, ? FROM source WHERE key = ?",
        )
        .bind(&m.url)
        .bind(m.title.clone().unwrap_or_default())
        .bind(&key)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE manga SET title = ? WHERE source_key = ?")
            .bind(m.title.clone().unwrap_or_default())
            .bind(&m.url)
            .execute(&mut *tx)
            .await?;
        let manga_id: (i64,) =
            sqlx::query_as("SELECT id FROM manga WHERE source_key = ?")
                .bind(&m.url)
                .fetch_one(&mut *tx)
                .await?;
        report.manga_imported += 1;

        for ch in &m.chapters {
            let number = ch.chapter_number.unwrap_or(0.0).to_string();
            let res = sqlx::query(
                "INSERT OR IGNORE INTO chapter (manga_id, source_key, number) VALUES (?, ?, ?)",
            )
            .bind(manga_id.0)
            .bind(&ch.url)
            .bind(&number)
            .execute(&mut *tx)
            .await?;
            if res.rows_affected() > 0 {
                report.chapters_imported += 1;
            }
        }

        // categories on the manga are INDICES into BackupCategory (Mihon semantics)
        for idx in &m.categories {
            if let Some(cat) = backup.backup_categories.get(*idx as usize) {
                let res = sqlx::query(
                    "INSERT OR IGNORE INTO manga_category (manga_id, category_id)                      SELECT ?, id FROM category WHERE name = ?",
                )
                .bind(manga_id.0)
                .bind(&cat.name)
                .execute(&mut *tx)
                .await?;
                if res.rows_affected() > 0 {
                    report.memberships_imported += 1;
                }
            }
        }
    }
    tx.commit().await?;
    Ok(report)
}

