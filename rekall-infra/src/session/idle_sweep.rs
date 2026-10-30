use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

/// Runs `pass` on the manager every `every` until it answers `false` or the manager is
/// dropped. The loop holds the manager weakly, so it never keeps the application alive.
pub fn spawn_idle_sweep<Manager, Pass, Sweeping>(manager: &Arc<Manager>, every: Duration, pass: Pass)
where
    Manager: Send + Sync + 'static,
    Pass: Fn(Arc<Manager>) -> Sweeping + Send + 'static,
    Sweeping: Future<Output = bool> + Send + 'static,
{
    let this = Arc::downgrade(manager);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + every, every);
        loop {
            interval.tick().await;
            let Some(manager) = this.upgrade() else { return };
            if !pass(manager).await {
                return;
            }
        }
    });
}

#[cfg(test)]
#[path = "../../tests/unit/session/idle_sweep_tests.rs"]
mod tests;
