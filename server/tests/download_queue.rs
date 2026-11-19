//! Wave 15 gate: the download queue is a DB-backed FIFO; the worker contains per-task
//! failures (state='failed' + error text) and keeps draining; done work is never redone.

use koma_server::db;
use koma_server::download::{enqueue, run_queue_once, DownloadError, QueueRunner, QueueTask};
use std::sync::atomic::{AtomicU32, Ordering};

static SERIAL: AtomicU32 = AtomicU32::new(0);

async fn seeded() -> koma_server::DbPool {
    let root = db::temp_test_dir("wave15");
    std::fs::create_dir_all(&root).expect("root");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    sqlx::query("INSERT INTO source (id, key, name) VALUES (1, 'mangadex', 'MangaDex')")
        .execute(&pool)
        .await
        .expect("source");
    sqlx::query("INSERT INTO manga (id, source_id, source_key, title) VALUES (1, 1, 'm1', 'M')")
        .execute(&pool)
        .await
        .expect("manga");
    for (cid, ext) in [(1, "c1"), (2, "c2"), (3, "c3"), (4, "c4")] {
        sqlx::query(
            "INSERT INTO chapter (id, manga_id, source_key, number, storage_dir) VALUES (?, 1, ?, '1', NULL)",
        )
        .bind(cid)
        .bind(ext)
        .execute(&pool)
        .await
        .expect("chapter");
    }
    pool
}

async fn state_of(pool: &koma_server::DbPool, id: i64) -> (String, Option<String>) {
    sqlx::query_as("SELECT state, state_error FROM chapter WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("row")
}

/// Deterministic runner: fails for the chapter ids in FAIL_IDS, records FIFO visit order.
struct TestRunner {
    fail_ids: &'static [i64],
    visited: std::sync::Mutex<Vec<i64>>,
}

impl QueueRunner for TestRunner {
    async fn run(&self, task: &QueueTask) -> Result<usize, DownloadError> {
        self.visited.lock().unwrap().push(task.chapter_row_id);
        if self.fail_ids.contains(&task.chapter_row_id) {
            Err(DownloadError::PageTooLarge(1))
        } else {
            Ok(3)
        }
    }
}

async fn order(pool: &koma_server::DbPool, id: i64, at: i64) {
    sqlx::query("UPDATE chapter SET queued_at = ? WHERE id = ?")
        .bind(at)
        .bind(id)
        .execute(pool)
        .await
        .expect("ordering");
}

#[tokio::test]
async fn fifo_order_and_failure_containment() {
    let pool = seeded().await;
    // Enqueue out of order (c3, c1, c2); enqueues land in the same second, so make the
    // FIFO order explicit: c3=1000, c1=1001, c2=1002. c4 stays unqueued (pending).
    enqueue(&pool, 3).await.expect("enqueue");
    enqueue(&pool, 1).await.expect("enqueue");
    enqueue(&pool, 2).await.expect("enqueue");
    order(&pool, 3, 1000).await;
    order(&pool, 1, 1001).await;
    order(&pool, 2, 1002).await;

    let runner = TestRunner { fail_ids: &[2], visited: std::sync::Mutex::new(vec![]) };
    let (ok, failed) = run_queue_once(&pool, &runner).await.expect("pass");
    assert_eq!((ok, failed), (2, 1), "3 processed: c1 ok, c2 failed-and-contained, c3 ok");

    assert_eq!(&*runner.visited.lock().unwrap(), &[3, 1, 2], "FIFO by enqueue time");

    // States after one pass:
    assert_eq!(state_of(&pool, 3).await.0, "downloaded");
    let (s1, _) = state_of(&pool, 1).await;
    assert_eq!(s1, "downloaded");
    let (s2, err2) = state_of(&pool, 2).await;
    assert_eq!(s2, "failed");
    assert!(err2.is_some(), "failure must carry its error text");
    assert_eq!(state_of(&pool, 4).await.0, "pending", "unqueued row stays untouched");
}

#[tokio::test]
async fn drained_work_is_not_redone() {
    let pool = seeded().await;
    enqueue(&pool, 1).await.expect("enqueue");
    let runner = TestRunner { fail_ids: &[], visited: std::sync::Mutex::new(vec![]) };
    let (ok, failed) = run_queue_once(&pool, &runner).await.expect("pass");
    assert_eq!((ok, failed), (1, 0));

    let (ok, failed) = run_queue_once(&pool, &runner).await.expect("second pass");
    assert_eq!((ok, failed), (0, 0));
    assert_eq!(runner.visited.lock().unwrap().len(), 1, "downloaded work is not redone");
}

#[tokio::test]
async fn new_rows_start_pending_and_unqueued_rows_are_untouched() {
    let pool = seeded().await;
    for id in 1..=4 {
        assert_eq!(state_of(&pool, id).await.0, "pending");
    }
    let runner = TestRunner { fail_ids: &[], visited: std::sync::Mutex::new(vec![]) };
    let (ok, failed) = run_queue_once(&pool, &runner).await.expect("pass");
    assert_eq!((ok, failed), (0, 0));
    assert_eq!(runner.visited.lock().unwrap().len(), 0, "worker touches only queued rows");
}
