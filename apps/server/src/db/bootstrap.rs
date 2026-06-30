use ::redis::aio::ConnectionManager;
use sqlx::PgPool;
use tracing::info;

use crate::common::config::AppConfig;
use crate::AppError;

use super::{migrate, pool, redis_client};

/// Live database and cache connections initialized at startup.
pub struct DatabaseBundle {
    pub pool: PgPool,
    pub redis: ConnectionManager,
}

/// Runs database migrations without starting the HTTP server.
///
/// # Errors
///
/// Returns an error if PostgreSQL connection or migration fails.
pub async fn run_migrations_only(config: &AppConfig) -> Result<(), AppError> {
    info!("Running database migrations");
    let pg_pool = pool::create_pool(&config.database_url).await?;
    pool::verify(&pg_pool).await?;
    migrate::run_migrations(&pg_pool).await?;
    info!("Database migrations complete");
    Ok(())
}

/// Connects to PostgreSQL and Redis, optionally running migrations.
///
/// # Errors
///
/// Returns an error if connection or migration setup fails.
pub async fn connect(config: &AppConfig) -> Result<DatabaseBundle, AppError> {
    info!("Connecting to PostgreSQL");
    let pg_pool = pool::create_pool(&config.database_url).await?;
    pool::verify(&pg_pool).await?;

    if config.run_migrations {
        migrate::run_migrations(&pg_pool).await?;
    }

    info!("Connecting to Redis");
    let redis_conn = redis_client::create_redis(&config.redis_url).await?;
    let mut redis_check = redis_conn.clone();
    redis_client::ping(&mut redis_check).await?;

    info!("All infrastructure connections established");
    Ok(DatabaseBundle {
        pool: pg_pool,
        redis: redis_conn,
    })
}
