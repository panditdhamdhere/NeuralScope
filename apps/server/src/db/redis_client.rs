use ::redis::aio::ConnectionManager;
use ::redis::{cmd, Client};

use crate::AppError;

/// Creates a multiplexed Redis connection manager.
///
/// # Errors
///
/// Returns an error if Redis is unreachable.
pub async fn create_redis(redis_url: &str) -> Result<ConnectionManager, AppError> {
    let client =
        Client::open(redis_url).map_err(|e| AppError::Config(format!("Invalid REDIS_URL: {e}")))?;

    ConnectionManager::new(client)
        .await
        .map_err(|e| AppError::External(format!("Redis connection failed: {e}")))
}

/// Verifies Redis connectivity with a PING command.
///
/// # Errors
///
/// Returns an error if the ping fails.
pub async fn ping(redis: &mut ConnectionManager) -> Result<(), AppError> {
    let _: String = cmd("PING")
        .query_async(redis)
        .await
        .map_err(|e| AppError::External(format!("Redis ping failed: {e}")))?;
    Ok(())
}
