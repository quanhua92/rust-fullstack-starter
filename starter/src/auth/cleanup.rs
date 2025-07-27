use crate::{
    auth::services,
    types::{DbPool, Result},
};
use tokio::time::{Duration, interval};
use tracing::{error, info};

/// Background job to clean up expired sessions
pub async fn session_cleanup_job(pool: DbPool) {
    let mut interval = interval(Duration::from_secs(3600)); // Run every hour

    loop {
        interval.tick().await;

        match cleanup_expired_sessions(&pool).await {
            Ok(count) => {
                if count > 0 {
                    info!("Cleaned up {} expired sessions", count);
                }
            }
            Err(e) => {
                error!("Failed to cleanup expired sessions: {}", e);
            }
        }
    }
}

/// Clean up expired sessions
async fn cleanup_expired_sessions(pool: &DbPool) -> Result<u64> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;
    services::cleanup_expired_sessions(&mut conn).await
}
