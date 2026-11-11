use rekall_core::connection::TaskTrackerService;
use rekall_core::task::TaskContentChangedEvent;
use rekall_core::{EventBus, PublishedEvent};
use rekall_kernel::Id;
use tokio::sync::broadcast::error::RecvError;
use tracing::warn;

/// Writes a task to its project's task manager once the transaction that created it or changed its
/// title or description has committed. Tasks are written one after another, in the order they
/// changed, so an edit never overtakes the creation it follows.
pub struct TaskTrackerRunner;

impl TaskTrackerRunner {
    /// Listens until the event bus closes.
    pub fn start(events: &EventBus, tracker: TaskTrackerService) {
        let mut bus = events.subscribe();
        tokio::spawn(async move {
            loop {
                match bus.recv().await {
                    Ok(event) => {
                        if let Some(task_id) = changed_task(&event) {
                            push(&tracker, task_id).await;
                        }
                    }
                    Err(RecvError::Lagged(missed)) => warn!(missed, "The task manager's runner missed events; the tasks they named are written on their next change"),
                    Err(RecvError::Closed) => return,
                }
            }
        });
    }
}

fn changed_task(event: &PublishedEvent) -> Option<Id> {
    event.downcast::<TaskContentChangedEvent>().map(|changed| changed.task_id)
}

async fn push(tracker: &TaskTrackerService, task_id: Id) {
    if let Err(failure) = tracker.push(task_id).await {
        warn!(%failure, %task_id, "A task could not be written to the project's task manager");
    }
}

#[cfg(test)]
#[path = "../../tests/unit/tracker/task_tracker_runner_tests.rs"]
mod tests;
