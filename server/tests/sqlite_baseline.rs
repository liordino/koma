//! Wave 2 gate: the SQLite operational baseline is real (decision 8).
//!
//! Proves connection-init actually turns on WAL, synchronous=NORMAL, FK enforcement and
//! busy_timeout — and that FK enforcement rejects an orphaned insert.

use koma_server::db;

async fn pragma_i32(pool: &sqlx::Pool<sqlx::Sqlite>, pragma: &str) -> i32 {
    let row: (i32,) = sqlx::query_as(&format!("PRAGMA {pragma}"))
        .fetch_one(pool)
        .await
        .expect(pragma);
    row.0
}

async fn pragma_str(pool: &sqlx::Pool<sqlx::Sqlite>, pragma: &str) -> String {
    let row: (String,) = sqlx::query_as(&format!("PRAGMA {pragma}"))
        .fetch_one(pool)
        .await
        .expect(pragma);
    row.0
}

#[tokio::test]
async fn baseline_pragmas_and_fk_enforcement() {
    let url = format!(
        "sqlite://{}?mode=rwc",
        std::env::temp_dir()
            .join(format!("koma-wave2-{}.db", std::process::id()))
            .display()
    );
    let pool = db::connect(&url).await.expect("connect");

    db::migrate(&pool).await.expect("migrate");

    // Baseline pragmas: journal_mode returns TEXT; synchronous NORMAL = 1, FK ON = 1, busy_timeout ms.
    assert_eq!(
        pragma_str(&pool, "journal_mode").await,
        "wal",
        "journal_mode must be WAL"
    );
    assert_eq!(
        pragma_i32(&pool, "synchronous").await,
        1,
        "synchronous must be NORMAL"
    );
    assert_eq!(
        pragma_i32(&pool, "foreign_keys").await,
        1,
        "foreign_keys must be ON"
    );
    assert_eq!(
        pragma_i32(&pool, "busy_timeout").await,
        5_000,
        "busy_timeout must be set"
    );

    // Orphan rejection — the corruption class this baseline exists to prevent:
    // a chapter referencing a nonexistent manga must fail.
    let orphan =
        sqlx::query("INSERT INTO chapter (manga_id, source_key, number) VALUES (99999, 'k', 'c1')")
            .execute(&pool)
            .await;
    let err = orphan.expect_err("orphan insert must be rejected by FK enforcement");
    assert!(
        err.to_string().to_lowercase().contains("foreign key"),
        "expected FK violation, got: {err}"
    );

    // And the same insert with a real parent must succeed.
    sqlx::query("INSERT INTO source (key, name) VALUES ('mangadex', 'MangaDex')")
        .execute(&pool)
        .await
        .expect("seed source");
    sqlx::query("INSERT INTO manga (source_id, source_key, title) VALUES (1, 'k', 'Test Manga')")
        .execute(&pool)
        .await
        .expect("seed manga");
    sqlx::query("INSERT INTO chapter (manga_id, source_key, number) VALUES (1, 'k2', 'c1')")
        .execute(&pool)
        .await
        .expect("valid chapter insert must pass");

    let _ = std::fs::remove_file(
        std::env::temp_dir().join(format!("koma-wave2-{}.db", std::process::id())),
    );
}
