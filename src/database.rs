pub mod document;
pub mod url;
pub mod visit;

use sqlx::ConnectOptions;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

use crate::settings::DatabaseSettings;

/// Instantiate database connection pool according to `settings`.
/// Will create database and/or apply migrations as necessary.
pub(crate) async fn create_pool(settings: DatabaseSettings) -> sqlx::Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_url(&settings.url)?
        .create_if_missing(true)
        .optimize_on_close(true, None);
    let pool = SqlitePool::connect_with(opts).await?;

    // Make sure database schema is up to date
    if let Some(migrations_source) = settings.migrations {
        Migrator::new(migrations_source).await?.run(&pool).await?;
    }

    Ok(pool)
}
