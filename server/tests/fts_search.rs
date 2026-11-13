//! Wave 4 gate: trigram FTS5 works on the sqlx-linked SQLite build — substring search must
//! match "One Piece" from the query "piece" (decision: Trigram FTS, .dojo/CONTEXT.md).

use koma_server::db;

async fn seed_pool() -> koma_server::DbPool {
    static SERIAL: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = SERIAL.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let url = format!(
        "sqlite://{}?mode=rwc",
        std::env::temp_dir()
            .join(format!("koma-wave4-{}-{n}.db", std::process::id()))
            .display()
    );
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    pool
}

async fn add_manga(pool: &koma_server::DbPool, title: &str, key: &str) -> i64 {
    sqlx::query("INSERT INTO source (key, name) VALUES ('mangadex', 'MangaDex')")
        .execute(pool)
        .await
        .expect("source");
    let r = sqlx::query("INSERT INTO manga (source_id, source_key, title) VALUES (1, ?, ?)")
        .bind(key)
        .bind(title)
        .execute(pool)
        .await
        .expect("manga");
    r.last_insert_rowid()
}

async fn search(pool: &koma_server::DbPool, q: &str) -> Vec<i64> {
    sqlx::query_as::<_, (i64,)>("SELECT rowid FROM manga_fts WHERE manga_fts MATCH ?")
        .bind(q)
        .fetch_all(pool)
        .await
        .expect("fts query")
        .into_iter()
        .map(|(id,)| id)
        .collect()
}

#[tokio::test]
async fn substring_query_matches_title_via_trigram() {
    let pool = seed_pool().await;
    let id = add_manga(&pool, "One Piece", "one-piece-k").await;
    let hits = search(&pool, "piece").await;
    assert_eq!(
        hits,
        vec![id],
        "'piece' must match 'One Piece' via trigram FTS"
    );
}

#[tokio::test]
async fn no_false_match_on_unrelated_title() {
    let pool = seed_pool().await;
    add_manga(&pool, "Vinland Saga", "vinland-k").await;
    let hits = search(&pool, "piece").await;
    assert!(hits.is_empty(), "'piece' must not match 'Vinland Saga'");
}

#[tokio::test]
async fn title_update_stays_in_sync() {
    let pool = seed_pool().await;
    let id = add_manga(&pool, "Berserk", "berserk-k").await;
    sqlx::query("UPDATE manga SET title = 'One Piece: New Dawn' WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .expect("update");
    let hits = search(&pool, "piece").await;
    assert_eq!(hits, vec![id], "triggered sync must reflect title updates");
}
