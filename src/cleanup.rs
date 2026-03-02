use sqlx::PgPool;
use tokio::time::{Duration, Instant, MissedTickBehavior, interval_at};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::db::scans;

/// Spawn the hourly retention cleanup task.
///
/// Deletes expired completed/failed scans (with 24h grace period beyond expires_at).
/// Registered with the TaskTracker so graceful shutdown waits for any in-progress DELETE.
/// First tick is deferred by 1 hour — no cleanup runs on startup/deploy.
pub fn spawn_cleanup_task(
    pool: PgPool,
    task_tracker: &TaskTracker,
    shutdown_token: CancellationToken,
) {
    task_tracker.spawn(async move {
        let period = Duration::from_secs(3600);
        let mut interval = interval_at(Instant::now() + period, period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    run_cleanup(&pool).await;
                }
                _ = shutdown_token.cancelled() => {
                    tracing::info!("Retention cleanup task shutting down");
                    break;
                }
            }
        }
    });
}

async fn run_cleanup(pool: &PgPool) {
    let anon = match scans::delete_expired_scans_by_tier(pool, "free").await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!(error = %e, "retention_cleanup_failed");
            return; // Log error, wait for next hourly tick — no immediate retry
        }
    };

    let dev = match scans::delete_expired_scans_by_tier(pool, "authenticated").await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!(error = %e, "retention_cleanup_failed");
            return;
        }
    };

    tracing::info!(
        anonymous_deleted = anon,
        developer_deleted = dev,
        total_deleted = anon + dev,
        "retention_cleanup"
    );
}
