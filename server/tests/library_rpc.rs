//! Wave 12 gate: library/category RPCs — service-level calls against real DB rows, plus one
//! router-level gRPC-Web call proving the RPC rides the bearer middleware.

use koma_server::auth::{hash_token, UserId};
use koma_server::db;
use koma_server::library::{LibraryDb, LibraryService};
use koma_server::pb as pb_types;
use tonic::Request;


async fn seeded() -> (koma_server::DbPool, LibraryDb) {
    let root = db::temp_test_dir("wave12");
    std::fs::create_dir_all(&root).expect("root");
    let url = format!("sqlite://{}?mode=rwc", root.join("koma.db").display());
    let pool = db::connect(&url).await.expect("connect");
    db::migrate(&pool).await.expect("migrate");
    koma_server::auth::seed_owner(&pool, "t").await.expect("seed");
    sqlx::query("INSERT INTO source (id, key, name) VALUES (1, 'mangadex', 'MangaDex')")
        .execute(&pool)
        .await
        .expect("source");
    (pool.clone(), LibraryDb { pool })
}

fn user_req<T>(msg: T) -> Request<T> {
    let mut r = Request::new(msg);
    r.extensions_mut().insert(UserId(1));
    r
}

#[tokio::test]
async fn create_category_then_add_manga_into_it() {
    let (_pool, lib) = seeded().await;

    let cat = lib
        .create_category(Request::new(pb_types::CreateCategoryRequest {
            name: "Reading".into(),
        }))
        .await
        .expect("create category")
        .into_inner()
        .category
        .expect("category in response");
    assert!(cat.id > 0);
    assert!(!cat.delete_after_read);

    // Idempotent upsert: same name must not duplicate.
    let cat2 = lib
        .create_category(Request::new(pb_types::CreateCategoryRequest {
            name: "Reading".into(),
        }))
        .await
        .expect("idempotent create")
        .into_inner()
        .category
        .expect("category in response");
    assert_eq!(cat.id, cat2.id);

    let cats = lib
        .list_categories(Request::new(pb_types::ListCategoriesRequest {}))
        .await
        .expect("list")
        .into_inner();
    assert_eq!(cats.items.len(), 1);

    let added = lib
        .add_manga(user_req(pb_types::AddMangaRequest {
            source_key: "mangadex".into(),
            external_id: "one-piece-ext".into(),
            title: "One Piece".into(),
            cover_url: String::new(),
            description: "Pirates".into(),
            category_id: cat.id,
        }))
        .await
        .expect("add")
        .into_inner()
        .manga
        .expect("manga in response");
    assert!(added.id > 0);
    assert_eq!(added.source_key, "mangadex");

    // Re-adding the same external_id must not duplicate the row.
    let again = lib
        .add_manga(user_req(pb_types::AddMangaRequest {
            source_key: "mangadex".into(),
            external_id: "one-piece-ext".into(),
            title: "One Piece".into(),
            cover_url: String::new(),
            description: String::new(),
            category_id: 0,
        }))
        .await
        .expect("idempotent add")
        .into_inner()
        .manga
        .expect("manga in response");
    assert_eq!(added.id, again.id);

    // By-category view contains it; the all view too.
    let in_cat = lib
        .list_library(Request::new(pb_types::ListLibraryRequest { category_id: cat.id }))
        .await
        .expect("list cat")
        .into_inner();
    assert_eq!(in_cat.items.len(), 1);
    let all = lib
        .list_library(Request::new(pb_types::ListLibraryRequest { category_id: 0 }))
        .await
        .expect("list all")
        .into_inner();
    assert_eq!(all.items.len(), 1);

    let got = lib
        .get_manga(Request::new(pb_types::GetMangaRequest { id: added.id }))
        .await
        .expect("get")
        .into_inner()
        .manga
        .expect("manga in response");
    assert_eq!(got.title, "One Piece");
    assert!(lib
        .get_manga(Request::new(pb_types::GetMangaRequest { id: 987654 }))
        .await
        .is_err());

    // Unknown source in AddManga → not_found.
    let bad = lib
        .add_manga(user_req(pb_types::AddMangaRequest {
            source_key: "unknown-src".into(),
            external_id: "x".into(),
            title: "T".into(),
            cover_url: String::new(),
            description: String::new(),
            category_id: 0,
        }))
        .await;
    assert!(bad.is_err());
}

#[tokio::test]
async fn upsert_progress_idempotent_and_clamped() {
    let (pool, lib) = seeded().await;
    sqlx::query("INSERT INTO manga (id, source_id, source_key, title) VALUES (1, 1, 'm-ext', 'T')")
        .execute(&pool)
        .await
        .expect("manga");
    sqlx::query("INSERT INTO chapter (id, manga_id, source_key, number) VALUES (1, 1, 'c-ext', '1')")
        .execute(&pool)
        .await
        .expect("chapter");

    let up = |page: i32, ratio: f64| {
        lib.upsert_progress(user_req(pb_types::UpsertProgressRequest {
            chapter_id: 1,
            page_n: page,
            scroll_ratio: ratio,
        }))
    };
    up(2, 0.5).await.expect("first");
    up(5, 1.5).await.expect("second, out-of-range ratio clamps");
    up(5, 0.9).await.expect("third");

    let row: (i32, f64) = sqlx::query_as("SELECT page_n, scroll_ratio FROM reading_progress WHERE user_id = 1 AND chapter_id = 1")
        .fetch_one(&pool)
        .await
        .expect("progress row");
    assert_eq!(row.0, 5);
    assert!((row.1 - 0.9).abs() < f64::EPSILON, "ratio clamped to <=1.0 and updated to 0.9");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reading_progress")
            .fetch_one(&pool)
            .await
            .expect("count"),
        1,
        "upsert semantics: one row per (user, chapter)"
    );
}

/// gRPC-Web frame: flag byte + 4-byte BE length + message bytes.
fn grpc_frame(msg: &[u8]) -> Vec<u8> {
    let mut f = vec![0u8];
    f.extend((msg.len() as u32).to_be_bytes());
    f.extend(msg);
    f
}

#[tokio::test]
async fn router_level_create_category_through_bearer_middleware() {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    let (pool, _lib) = seeded().await;
    // Token "t" is the seeded owner's; its hash must be what /pages-style auth stores.
    let _ = hash_token("t");

    let router = koma_server::build_router(pool, &["http://localhost:5173"], std::env::temp_dir(), koma_server::fetch::test_client());

    let body = pb_types::CreateCategoryRequest { name: "FromWire".into() };
    let mut bytes = Vec::new();
    prost::Message::encode(&body, &mut bytes).expect("encode");

    // No token → 401 before any RPC work:
    let req = Request::builder()
        .method("POST")
        .uri("/koma.v1.LibraryService/CreateCategory")
        .header("content-type", "application/grpc-web+proto")
        .body(Body::from(grpc_frame(&bytes)))
        .unwrap();
    let res = router.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "RPC requires bearer");

    // With token → OK, and the row exists in the DB:
    let req = Request::builder()
        .method("POST")
        .uri("/koma.v1.LibraryService/CreateCategory")
        .header("content-type", "application/grpc-web+proto")
        .header(header::AUTHORIZATION, "Bearer t")
        .body(Body::from(grpc_frame(&bytes)))
        .unwrap();
    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let ct = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(ct.starts_with("application/grpc"), "grpc response, got {ct}");
}
