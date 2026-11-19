//! Wave 17 gate: mark-read moves delete_after_read chapters' files OUT of the serving path
//! (trash sibling dir), records retention; non-delete categories keep files; re-download resets.

use koma_server::db;
use koma_server::trash::mark_read;

async fn seeded(with_delete_category: bool) -> (koma_server::DbPool, std::path::PathBuf, i64) {
    let root = db::temp_test_dir("wave17");
    std::fs::create_dir_all(&root).expect("root");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    sqlx::query("INSERT INTO source (id, key, name) VALUES (1, 'mangadex', 'MangaDex')")
        .execute(&pool).await.expect("source");
    sqlx::query("INSERT INTO manga (id, source_id, source_key, title) VALUES (1, 1, 'm-ext', 'M')")
        .execute(&pool).await.expect("manga");
    if with_delete_category {
        sqlx::query("INSERT INTO category (id, name, delete_after_read) VALUES (1, 'consumed', 1)")
            .execute(&pool).await.expect("category");
        sqlx::query("INSERT INTO manga_category (manga_id, category_id) VALUES (1, 1)")
            .execute(&pool).await.expect("membership");
    }
    sqlx::query("INSERT INTO chapter (id, manga_id, source_key, number, storage_dir) VALUES (1, 1, 'ch-ext', '1', 'mangadex/chapters/ch-ext')")
        .execute(&pool).await.expect("chapter");
    let cid: i64 = 1;
    (pool, root, cid)
}

fn plant_files(storage_root: &std::path::Path) -> std::path::PathBuf {
    let dir = storage_root.join("mangadex/chapters/ch-ext");
    std::fs::create_dir_all(&dir).expect("dir");
    for n in 0..3u32 {
        std::fs::write(dir.join(format!("page_{n}.jpg")), b"jpg").expect("file");
    }
    std::path::Path::new(storage_root).join("trash")
}

#[tokio::test]
async fn delete_after_read_moves_files_to_trash() {
    let (pool, storage_root, cid) = seeded(true).await;
    let storage_root = std::path::Path::new(&storage_root).to_path_buf();
    std::fs::create_dir_all(storage_root.join("storage")).expect("storage");
    let storage_dir = storage_root.join("storage");
    // mark_read uses trash_root().parent() as the base — pass trash path = storage/storage/trash
    let trash: std::path::PathBuf = storage_dir.join("trash");
    let files = storage_dir.join("mangadex/chapters/ch-ext");
    std::fs::create_dir_all(&files).expect("chapter dir");
    std::fs::write(files.join("page_0.jpg"), b"x").expect("file");
    let (moved, _retention) = mark_read(&pool, cid, &trash).await.expect("mark");
    assert!(moved);
    assert!(!files.join("page_0.jpg").exists(), "originals left the serving path");
    // row: downloaded=0, pending, trash_dir recorded
    let (dl, sd, tdir): (i32, Option<String>, Option<String>) =
        sqlx::query_as("SELECT downloaded, storage_dir, trash_dir FROM chapter WHERE id = 1")
            .fetch_one(&pool).await.expect("row");
    assert_eq!(dl, 0);
    assert!(sd.is_none());
    assert!(tdir.is_some());
}

#[tokio::test]
async fn non_delete_category_keeps_files() {
    let (pool, storage_root, cid) = seeded(false).await;
    let storage_root = std::path::Path::new(&storage_root).to_path_buf();
    let storage_dir = storage_root.join("storage");
    std::fs::create_dir_all(storage_dir.join("mangadex/chapters/ch-ext")).expect("dir");
    std::fs::write(storage_dir.join("mangadex/chapters/ch-ext/page_0.jpg"), b"x").expect("file");
    let trash: std::path::PathBuf = storage_dir.join("trash");
    let (moved, retention) = mark_read(&pool, cid, &trash).await.expect("mark");
    assert!(!moved, "no delete_after_read category → files stay");
    assert!(storage_dir.join("mangadex/chapters/ch-ext/page_0.jpg").exists());
    assert!(retention > 0, "retention timestamp still recorded");
}
