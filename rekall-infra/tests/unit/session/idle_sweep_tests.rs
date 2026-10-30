use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::spawn_idle_sweep;

const EVERY: Duration = Duration::from_millis(20);

#[tokio::test]
async fn the_pass_runs_every_period_until_it_answers_false() {
    let passes = Arc::new(AtomicUsize::new(0));
    let manager = Arc::new(passes.clone());

    spawn_idle_sweep(&manager, EVERY, |counter: Arc<Arc<AtomicUsize>>| async move {
        counter.fetch_add(1, Ordering::SeqCst) + 1 < 2
    });

    tokio::time::sleep(EVERY * 10).await;
    assert_eq!(passes.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn the_loop_ends_when_the_manager_is_dropped() {
    let passes = Arc::new(AtomicUsize::new(0));
    let manager = Arc::new(passes.clone());

    spawn_idle_sweep(&manager, EVERY, |counter: Arc<Arc<AtomicUsize>>| async move {
        counter.fetch_add(1, Ordering::SeqCst);
        true
    });
    drop(manager);

    tokio::time::sleep(EVERY * 5).await;
    assert_eq!(passes.load(Ordering::SeqCst), 0);
}
