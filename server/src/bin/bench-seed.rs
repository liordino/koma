//! Bench seed helper: chapter 1 in the DB + 10 small JPEG-ish page files in the storage root,
//! so the benchmark harness can pull /pages/1/0..9.

use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url =
        std::env::var("KOMA_DB_URL").unwrap_or_else(|_| "sqlite://koma.db?mode=rwc".to_string());
    let storage_root = PathBuf::from(
        std::env::var("KOMA_STORAGE").unwrap_or_else(|_| "./storage".into()),
    );
    let token = std::env::var("KOMA_TOKEN").unwrap_or_else(|_| "koma-dev-token".into());

    std::fs::create_dir_all(storage_root.join("pages"))?;
    for n in 0..10u32 {
        // A tiny valid-ish JPEG (SOI/EOI markers suffice for transport timing; decoding is
        // client-side and not part of this benchmark).
        std::fs::write(storage_root.join("pages").join(format!("page_{n}.jpg")), {
            let mut b = vec![0xFF, 0xD8, 0xFF, 0xE0];
            b.extend(std::iter::repeat_n(0xABu8, 64 * 1024));
            b.extend([0xFF, 0xD9]);
            b
        })?;
    }

    let pool = koma_server::db::connect(&db_url).await?;
    koma_server::db::migrate(&pool).await?;
    koma_server::auth::seed_owner(&pool, &token).await?;

    sqlx::query("INSERT OR IGNORE INTO source (id, key, name) VALUES (1, 'mangadex', 'MangaDex')")
        .execute(&pool)
        .await?;
    sqlx::query("INSERT OR IGNORE INTO manga (id, source_id, source_key, title) VALUES (1, 1, 'bench', 'Bench Manga')")
        .execute(&pool)
        .await?;
    sqlx::query(
        "INSERT OR IGNORE INTO chapter (id, manga_id, source_key, number, storage_dir) VALUES (1, 1, 'bench', '1', 'pages')",
    )
    .execute(&pool)
    .await?;

    println!("bench seed ok");
    Ok(())
}
