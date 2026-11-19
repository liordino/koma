//! Wave 16 gate: update pass is idempotent — a re-run adds nothing; novelty is counted.

use koma_server::db;
use koma_server::update::{run_update_pass, ChapterInfo, UpdatableSource};

struct MockSource {
    chapters: Vec<(&'static str, &'static str)>,
}

impl UpdatableSource for MockSource {
    async fn list_chapters(
        &self,
        _manga_external_id: &str,
        _limit: u32,
    ) -> Result<Vec<ChapterInfo>, koma_server::fetch::FetchError> {
        Ok(self
            .chapters
            .iter()
            .map(|(e, n)| ChapterInfo { external_id: e.to_string(), number: n.to_string() })
            .collect())
    }
}

async fn seeded() -> koma_server::DbPool {
    let root = db::temp_test_dir("wave16");
    std::fs::create_dir_all(&root).expect("dir");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    sqlx::query("INSERT INTO source (id, key, name) VALUES (1, 'mangadex', 'MangaDex')")
        .execute(&pool)
        .await
        .expect("source");
    sqlx::query("INSERT INTO manga (id, source_id, source_key, title) VALUES (1, 1, 'm-ext', 'M')")
        .execute(&pool)
        .await
        .expect("manga");
    pool
}

#[tokio::test]
async fn pass_adds_new_then_rerun_adds_zero() {
    let pool = seeded().await;
    let source = MockSource { chapters: vec![("ch-a", "1"), ("ch-b", "2")] };

    let added = run_update_pass(&pool, &source, "mangadex", 100).await.expect("pass 1");
    assert_eq!(added, 2, "both chapters are new");

    let added2 = run_update_pass(&pool, &source, "mangadex", 100).await.expect("pass 2");
    assert_eq!(added2, 0, "re-run must add nothing (UNIQUE novelty contract)");

    let added3 = run_update_pass(&pool, &source, "mangadex", 100).await.expect("pass 3");
    assert_eq!(added3, 0);
}

#[tokio::test]
async fn incremental_pass_adds_only_new_chapters() {
    let pool = seeded().await;
    let small = MockSource { chapters: vec![("ch-a", "1")] };
    assert_eq!(run_update_pass(&pool, &small, "mangadex", 50).await.expect("p1"), 1);

    // A source later publishes two more chapters; the pass must add exactly those.
    let big = MockSource { chapters: vec![("ch-a", "1"), ("ch-b", "2"), ("ch-c", "3")] };
    assert_eq!(run_update_pass(&pool, &big, "mangadex", 50).await.expect("p2"), 2);

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM chapter").fetch_one(&pool).await.expect("count");
    assert_eq!(count.0, 3, "no duplicates across passes");
}

#[tokio::test]
async fn pass_ignores_manga_of_other_sources() {
    let pool = seeded().await;
    sqlx::query("INSERT INTO source (id, key, name) VALUES (2, 'other', 'Other')")
        .execute(&pool)
        .await
        .expect("source2");
    sqlx::query("INSERT INTO manga (id, source_id, source_key, title) VALUES (2, 2, 'other-ext', 'O')")
        .execute(&pool)
        .await
        .expect("manga2");

    let source = MockSource { chapters: vec![("ch-x", "9")] };
    assert_eq!(run_update_pass(&pool, &source, "mangadex", 100).await.expect("pass"), 1);
    let rows: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chapter WHERE manga_id = 2")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(rows.0, 0, "pass scoped to source_key only");
}
