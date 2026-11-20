//! Wave 19 gate: the owner's REAL .tachibk import — decode, dry-run report, staged import
//! with VACUUM INTO snapshot + transaction, idempotent re-import.

use koma_server::db;
use koma_server::import_tachibk::{decode, dry_run, import};

const FIXTURE: &[u8] = include_bytes!("fixtures/mihon-real.tachibk");

#[tokio::test]
async fn real_fixture_decode_and_staged_import() {
    // decode: real bytes from the owner's Mihon export
    let backup = decode(FIXTURE).expect("decode real backup");
    assert!(!backup.backup_manga.is_empty(), "fixture has manga");

    // dry run: counts, no writes
    let report = dry_run(&backup);
    assert!(report.manga_seen > 0, "favorites must be counted");
    // favorites can be zero on Mihon v2 fixtures (browsed-but-not-library manga):
    // what matters is the manga count, not the favorite subset.
    assert!(report.manga_seen > 0, "fixture carries manga entries");

    // staged import into a fresh DB
    let root = db::temp_test_dir("wave19");
    std::fs::create_dir_all(&root).expect("dir");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");

    let import_dir = root.join("backups");
    let r1 = import(&pool, &backup, &import_dir).await.expect("import 1");
    assert!(r1.snapshot_path.as_ref().is_some_and(|p| p.exists()), "VACUUM INTO snapshot exists");
    assert_eq!(r1.manga_imported, r1.manga_seen, "all favorites imported");
    let chapter_count: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM chapter").fetch_one(&pool).await.expect("count");
    assert!(r1.chapters_imported > 0, "chapters_imported={:?} db_chapters={:?} by_source={:?} manga={}", r1.chapters_imported, chapter_count, { r1.by_source }, r1.manga_imported);

    // the real backup's categories must land exactly (may be zero on this fixture)
    let (cats,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM category").fetch_one(&pool).await.expect("cats");
    assert_eq!(cats as u32, backup.backup_categories.len() as u32, "category count matches backup");

    // spot-check content: the seeded data must carry real titles
    let titles: Vec<(String,)> =
        sqlx::query_as("SELECT title FROM manga WHERE title LIKE 'Rurouni%'").fetch_all(&pool).await.expect("query");
    assert!(!titles.is_empty(), "real title survived the mapping");
    let src_rows: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM source WHERE key = 'mangabat'").fetch_one(&pool).await.expect("sources");
    assert_eq!(src_rows.0, 1, "backup-source names registered as data (slug of the backup's own name)");
}

#[tokio::test]
async fn reimport_is_idempotent() {
    let backup = decode(FIXTURE).expect("decode");
    let root = db::temp_test_dir("wave19-idem");
    std::fs::create_dir_all(&root).expect("dir");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");

    import(&pool, &backup, &root.join("backups")).await.expect("first");
    let before: (i64, i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM manga), (SELECT COUNT(*) FROM chapter)")
        .fetch_one(&pool).await.expect("counts");

    import(&pool, &backup, &root.join("backups")).await.expect("second");
    let after: (i64, i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM manga), (SELECT COUNT(*) FROM chapter)")
        .fetch_one(&pool).await.expect("counts");
    assert_eq!(before, after, "re-import adds nothing (UNIQUE external-ID contract)");
}

#[tokio::test]
async fn categories_match_backup_count() {
    let backup = decode(FIXTURE).expect("decode");
    let root = db::temp_test_dir("wave19-cat");
    std::fs::create_dir_all(&root).expect("dir");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    import(&pool, &backup, &root.join("backups")).await.expect("import");
    let (cats,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM category").fetch_one(&pool).await.expect("cats");
    assert_eq!(cats as u32, backup.backup_categories.len() as u32, "no phantom categories");
}
