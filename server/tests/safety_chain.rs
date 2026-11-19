//! Wave 18 gate: the delete_after_read safety chain — a purge refuses to run unconfirmed or
//! without a fresh metadata backup, and deletes trash files + chapter rows only after both.

use koma_server::backup::{export_metadata, purge_category};
use koma_server::db;
use koma_server::trash::mark_read;

async fn seeded() -> (koma_server::DbPool, std::path::PathBuf, std::path::PathBuf) {
    let root = db::temp_test_dir("wave18");
    std::fs::create_dir_all(&root).expect("root");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    sqlx::query("INSERT INTO source (id, key, name) VALUES (1, 'mangadex', 'MangaDex')")
        .execute(&pool).await.expect("source");
    sqlx::query("INSERT INTO manga (id, source_id, source_key, title) VALUES (1, 1, 'm-ext', 'M')")
        .execute(&pool).await.expect("manga");
    sqlx::query("INSERT INTO category (id, name, delete_after_read) VALUES (1, 'consumed', 1)")
        .execute(&pool).await.expect("category");
    sqlx::query("INSERT INTO manga_category (manga_id, category_id) VALUES (1, 1)")
        .execute(&pool).await.expect("membership");
    sqlx::query("INSERT INTO chapter (id, manga_id, source_key, number, storage_dir) VALUES (1, 1, 'ch-ext', '1', 'mangadex/chapters/ch-ext')")
        .execute(&pool).await.expect("chapter");
    let storage = root.join("storage");
    std::fs::create_dir_all(storage.join("mangadex/chapters/ch-ext")).expect("chapter dir");
    std::fs::write(storage.join("mangadex/chapters/ch-ext/page_0.jpg"), b"x").expect("file");
    (pool, root.join("storage"), root.join("trash"))
}

/// mark_read runs the trash move (wave 17), pushing the chapter into purge scope.
async fn read_and_trash(pool: &koma_server::DbPool, trash: &std::path::Path) {
    let (moved, _) = mark_read(pool, 1, trash).await.expect("mark");
    assert!(moved, "fixture requires the trash move to happen");
}

#[tokio::test]
async fn backup_export_writes_real_metadata() {
    let (pool, _storage, _trash) = seeded().await;
    let backup_dir = db::temp_test_dir("wave18-backups");
    let (path, ts) = export_metadata(&pool, &backup_dir).await.expect("export");
    assert!(path.exists());
    let doc: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("read")).expect("json");
    assert_eq!(doc["exported_at"], ts);
    assert_eq!(doc["chapters"].as_array().expect("chapters array").len(), 1);
    assert_eq!(doc["mangas"].as_array().expect("mangas array").len(), 1);
}

#[tokio::test]
async fn purge_refuses_unconfirmed() {
    let (pool, _storage, trash) = seeded().await;
    read_and_trash(&pool, &trash).await;
    let err = purge_category(&pool, &trash, 1, false).await.expect_err("must refuse");
    assert!(err.to_string().contains("confirmation"), "got {err}");
    // nothing happened
    let rows: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chapter").fetch_one(&pool).await.expect("count");
    assert_eq!(rows.0, 1);
}

#[tokio::test]
async fn purge_refuses_stale_backup() {
    let (pool, _storage, trash) = seeded().await;
    // back up BEFORE the read/trash move → backup predates the newest trash move
    export_metadata(&pool, &db::temp_test_dir("wave18-b1")).await.expect("backup");
    read_and_trash(&pool, &trash).await;
    let err = purge_category(&pool, &trash, 1, true).await.expect_err("must refuse");
    assert!(err.to_string().contains("stale"), "got {err}");
    let rows: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chapter").fetch_one(&pool).await.expect("count");
    assert_eq!(rows.0, 1, "nothing purged");
}

#[tokio::test]
async fn purge_succeeds_after_fresh_backup_and_is_explicit() {
    let (pool, _storage, trash) = seeded().await;
    read_and_trash(&pool, &trash).await;

    // fresh backup AFTER the trash move → freshness gate satisfied
    let (path, _) = export_metadata(&pool, &db::temp_test_dir("wave18-b2")).await.expect("fresh backup");
    assert!(path.exists());

    // simulate the freshness bookkeeping the RPC flow does: export stamps category.last_backup_at
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    sqlx::query("UPDATE category SET last_backup_at = ? WHERE id = 1").bind(now).execute(&pool).await.expect("stamp");

    let purged = purge_category(&pool, &trash, 1, true).await.expect("purge");
    assert_eq!(purged, 1);
    // chapter row gone; trash files gone
    let rows: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chapter").fetch_one(&pool).await.expect("count");
    assert_eq!(rows.0, 0);
    assert!(!trash.join("mangadex").exists(), "trash dir removed");
}

#[tokio::test]
async fn purge_unknown_category_refused() {
    let (pool, _storage, trash) = seeded().await;
    let err = purge_category(&pool, &trash, 999, true).await.expect_err("unknown category");
    assert!(err.to_string().contains("not found"), "got {err}");
}
