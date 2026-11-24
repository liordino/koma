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
    // The shipped tier-A adapter self-registers (the registry is data, but the binary
    // owns the rows for adapters it compiles in).
    koma_server::source::seed_builtin_sources(&pool).await?;

    // Production outbound: MangaDex API + At-Home covers only (allowlist is the SSRF guard).
    let outbound = std::sync::Arc::new(
        koma_server::fetch::OutboundClient::new(
            vec!["mangadex.org".to_string(), "mangadex.network".to_string()],
            5,
            10,
            std::time::Duration::from_secs(1),
        )?,
    );

    let origin_refs: Vec<&str> = allowed_origins.iter().map(String::as_str).collect();
    let mdx_worker = koma_server::source::mangadex::MangaDexAdapter::with_client(
        outbound.clone(),
        &std::env::var("KOMA_MANGADEX_API_BASE")
            .unwrap_or_else(|_| "https://api.mangadex.org".to_string()),
    );
    let runner = koma_server::download::RealRunner {
        http: outbound.clone(),
        adapter: mdx_worker.clone(),
        pool: pool.clone(),
        storage_root: storage_root.clone(),
    };
    // Self-driving download queue (M2 wave 15's worker, now live in the binary):
    // a supervised pass every poll window — failures are contained per-task.
    {
        let worker_pool = pool.clone();
        tokio::spawn(async move {
            loop {
                match koma_server::download::run_queue_once(&worker_pool, &runner).await {
                    Ok(_) => {}
                    Err(e) => eprintln!("queue worker error: {e}"),
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
    }

    let app = build_router(pool, &origin_refs, storage_root, outbound);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("koma-server on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
