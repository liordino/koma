//! SQLite connection-init module — the operational baseline (decision 8, .dojo/CONTEXT.md).
//!
//! Every pool is created through this module so WAL, synchronous=NORMAL, foreign_keys=ON and
//! busy_timeout are set on every connection, everywhere, exactly once in the codebase.

use std::time::Duration;

pub type DbPool = sqlx::Pool<Sqlite>;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    Pool, Sqlite,
};

/// Open a pool with the Koma SQLite operational baseline applied per connection.
pub async fn connect(url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    let options = baseline_options(url)?;
    // Single-writer discipline (decision 8): one connection front line.
    sqlx::pool::PoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
}

/// The baseline connection options, split out so tests and the pool share one definition.
pub fn baseline_options(url: &str) -> Result<SqliteConnectOptions, sqlx::Error> {
    Ok(SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_millis(5_000)))
}

use std::str::FromStr;

/// Run all numbered migrations in order. Idempotent (sqlx tracks applied versions).
pub async fn migrate(pool: &Pool<Sqlite>) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Globally-unique temp dir for integration tests. Windows reuses pids aggressively, so
/// pid-based temp names collide with stale DBs from prior runs (observed as UNIQUE
/// constraint failures); nanos timestamps cannot collide. Callers remove the dir on exit
/// (or leave it — stale dirs are now harmless).
pub fn temp_test_dir(prefix: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("koma-test-{prefix}-{nanos}"))
}
