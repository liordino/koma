//! Wave 7 gate: path-safe /pages serving. The traversal regression test from day one (BRIEF.md
//! hard constraint): page requests must be DB-keyed, bounded, and cannot escape the storage root.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use tower::ServiceExt;

use koma_server::{db, pages};

async fn seed() -> (koma_server::DbPool, std::path::PathBuf, axum::Router) {
    let root: std::path::PathBuf = db::temp_test_dir("wave7");
    std::fs::create_dir_all(&root).expect("create storage root");

    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");

    // Every route — /pages included — is behind the bearer middleware (decision 24).
    koma_server::auth::seed_owner(&pool, "t").await.expect("seed owner");

    let router = koma_server::build_router(pool.clone(), &["http://localhost:5173"], root.clone());
    (pool, root, router)
}

async fn add_chapter(pool: &koma_server::DbPool, storage_dir: &str) -> i64 {
    sqlx::query("INSERT INTO source (key, name) VALUES ('mangadex', 'MangaDex')")
        .execute(pool)
        .await
        .expect("source");
    sqlx::query("INSERT INTO manga (source_id, source_key, title) VALUES (1, 'k', 'Test')")
        .execute(pool)
        .await
        .expect("manga");
    let r = sqlx::query("INSERT INTO chapter (manga_id, source_key, number, storage_dir) VALUES (1, 'ck', '1', ?)")
        .bind(storage_dir)
        .execute(pool)
        .await
        .expect("chapter");
    r.last_insert_rowid()
}

async fn request_page(router: axum::Router, chapter_id: i64, page: &str) -> axum::http::Response<Body> {
    let req = Request::builder()
        .method("GET")
        .uri(format!("/pages/{chapter_id}/{page}"))
        .header(header::AUTHORIZATION, "Bearer t")
        .body(axum::body::Body::empty())
        .unwrap();
    router.oneshot(req).await.unwrap()
}

fn write_page(root: &std::path::Path, dir: &str, page_n: u32, content: &[u8]) {
    std::fs::create_dir_all(root.join(dir)).expect("chapter dir");
    std::fs::write(root.join(dir).join(format!("page_{page_n}.jpg")), content).expect("page");
}

#[tokio::test]
async fn valid_page_request_serves_bytes() {
    let (pool, root, router) = seed().await;
    let id = add_chapter(&pool, "chap1").await;
    write_page(&root, "chap1", 0, b"JPEGDATA");
    let res = request_page(router, id, "0").await;
    assert_eq!(res.status(), StatusCode::OK);
    let ct = res.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
    assert_eq!(ct, "image/jpeg");
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), b"JPEGDATA");
}

#[tokio::test]
async fn traversal_storage_dir_is_rejected() {
    let (pool, _root, router) = seed().await;
    // Storage dir planted with traversal — must never resolve outside the root.
    let id = add_chapter(&pool, "../../etc").await;
    let res = request_page(router, id, "0").await;
    assert_ne!(
        res.status(),
        StatusCode::OK,
        "traversal storage_dir must not serve bytes"
    );
}

#[tokio::test]
async fn traversal_in_page_number_is_bounded() {
    let (pool, root, router) = seed().await;
    let id = add_chapter(&pool, "chapA").await;
    write_page(&root, "chapA", 0, b"X");
    // Out-of-range page indexes are rejected before any path is built.
    let res_max = request_page(router.clone(), id, "99999").await;
    assert_eq!(res_max.status(), StatusCode::BAD_REQUEST);
    // Negative and encoded traversal page values must not match a real file either.
    let res_neg = request_page(router, id, "4294967295").await; // u32::MAX as string
    assert_ne!(res_neg.status(), StatusCode::OK);
}

#[tokio::test]
async fn unknown_chapter_is_404_not_probe() {
    let (_pool, _root, router) = seed().await;
    let res = request_page(router, 424242, "0").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[test]
fn page_bound_constant_is_reasonable() {
    const { assert!(pages::MAX_PAGE_N < 100_000, "page bound sanity") }
}
