//! Koma server binary — boots the SQLite baseline, seeds the owner token, serves Axum.

use koma_server::{auth, build_router, db};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url =
        std::env::var("KOMA_DB_URL").unwrap_or_else(|_| "sqlite://koma.db?mode=rwc".to_string());
    let storage_root =
        PathBuf::from(std::env::var("KOMA_STORAGE").unwrap_or_else(|_| "./storage".into()));
    let addr = std::env::var("KOMA_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let token = std::env::var("KOMA_TOKEN").unwrap_or_else(|_| "koma-dev-token".into());
    let allowed_origins: Vec<String> = std::env::var("KOMA_ORIGINS")
        .map(|s| s.split(',').map(str::trim).map(String::from).collect())
        .unwrap_or_else(|_| vec!["http://localhost:5173".into()]);

    std::fs::create_dir_all(&storage_root)?;

    let pool = db::connect(&db_url).await?;
    db::migrate(&pool).await?;
    auth::seed_owner(&pool, &token).await?;

    let origin_refs: Vec<&str> = allowed_origins.iter().map(String::as_str).collect();
    let app = build_router(pool, &origin_refs, storage_root);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("koma-server on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
