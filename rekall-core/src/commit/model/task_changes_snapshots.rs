use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use rekall_kernel::{Id, RekallError, Result};

use super::task_changes_snapshot::TaskChangesSnapshot;

pub(in crate::commit) const SNAPSHOT_TTL: Duration = Duration::from_secs(5 * 60);
pub(in crate::commit) const MAX_SNAPSHOTS: usize = 16;

#[derive(Default)]
pub(in crate::commit) struct TaskChangesSnapshots {
    entries: VecDeque<TimedSnapshot>,
}

struct TimedSnapshot {
    created: Instant,
    snapshot: Arc<TaskChangesSnapshot>,
}

impl TaskChangesSnapshots {
    pub fn insert(&mut self, now: Instant, snapshot: TaskChangesSnapshot) {
        self.expire(now);
        if self.entries.len() == MAX_SNAPSHOTS {
            self.entries.pop_front();
        }
        self.entries.push_back(TimedSnapshot {
            created: now,
            snapshot: Arc::new(snapshot),
        });
    }

    pub fn require(&mut self, now: Instant, task_id: Id, id: Id) -> Result<Arc<TaskChangesSnapshot>> {
        self.expire(now);
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.snapshot.id == id)
            .ok_or_else(|| RekallError::conflict("The Changes snapshot has expired. Request a fresh file index."))?;
        if entry.snapshot.task_id != task_id {
            return Err(RekallError::not_found("Changes snapshot", id));
        }
        Ok(Arc::clone(&entry.snapshot))
    }

    fn expire(&mut self, now: Instant) {
        self.entries.retain(|entry| now.duration_since(entry.created) < SNAPSHOT_TTL);
    }
}
