use sqlx::PgPool;
use tracing::info;

use crate::AppError;

/// Applies pending SQLx migrations against the database.
///
/// # Errors
///
/// Returns an error if any migration fails to apply.
pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    info!("Running database migrations");
    sqlx::migrate!()
        .run(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Migration failed: {e}")))?;
    info!("Database migrations complete");
    Ok(())
}
