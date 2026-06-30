use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::AppError;

/// Creates a PostgreSQL connection pool with configured limits.
///
/// # Errors
///
/// Returns an error if the database is unreachable.
pub async fn create_pool(database_url: &str) -> Result<PgPool, AppError> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(AppError::from)
}

/// Verifies database connectivity with a lightweight query.
///
/// # Errors
///
/// Returns an error if the database is unreachable.
pub async fn verify(pool: &PgPool) -> Result<(), AppError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(AppError::from)?;
    Ok(())
}
