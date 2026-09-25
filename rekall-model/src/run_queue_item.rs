//! One task in the run queue, at a dense `position`, with what happened when its turn came.

use rekall_common::{jstr, Id, Instant};
use sea_orm::entity::prelude::*;

use crate::enums::RunQueueItemState;

pub const DETAIL_MAX: usize = 500;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "run_queue_item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub position: i32,
    pub state: RunQueueItemState,
    /// Why the item is where it is, in a sentence, when that is not obvious from the state.
    pub detail: Option<String>,
    pub started_at: Option<Instant>,
    pub finished_at: Option<Instant>,
    pub created_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::TaskId",
        to = "super::task::Column::Id",
        on_delete = "Cascade"
    )]
    Task,
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// `new RunQueueItem(task, position)`: waiting its turn.
    pub fn new(task_id: Id, position: i32, now: Instant) -> Self {
        Self {
            id: Id::random(),
            task_id,
            position,
            state: RunQueueItemState::Queued,
            detail: None,
            started_at: None,
            finished_at: None,
            created_at: now,
        }
    }

    /// `RunQueueItem.moveTo`: move with an optional reason. Starting stamps `started_at` the
    /// first time only, so a task resumed after a hold keeps the moment it first ran; settling
    /// stamps `finished_at`; going back to `QUEUED` clears it.
    pub fn move_to(&mut self, next: RunQueueItemState, reason: Option<&str>) {
        self.state = next;
        self.detail = reason.map(|r| truncate(jstr::strip(r)));
        let now = Instant::now();
        if next == RunQueueItemState::Running && self.started_at.is_none() {
            self.started_at = Some(now);
        }
        self.finished_at = if next.settled() { Some(now) } else { None };
    }
}

fn truncate(text: &str) -> String {
    if jstr::len(text) <= DETAIL_MAX {
        text.to_string()
    } else {
        format!("{}…", jstr::prefix(text, DETAIL_MAX - 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_stamps_once_settling_stamps_and_requeueing_clears() {
        let mut item = Model::new(Id::random(), 0, Instant::now());
        item.move_to(RunQueueItemState::Running, None);
        let started = item.started_at;
        assert!(started.is_some());
        item.move_to(RunQueueItemState::Queued, Some("  paused  "));
        assert_eq!(item.detail.as_deref(), Some("paused"));
        item.move_to(RunQueueItemState::Running, None);
        assert_eq!(item.started_at, started);
        item.move_to(RunQueueItemState::Finished, None);
        assert!(item.finished_at.is_some());
        let long = "x".repeat(600);
        item.move_to(RunQueueItemState::Failed, Some(&long));
        assert_eq!(jstr::len(item.detail.as_deref().unwrap()), DETAIL_MAX);
    }
}
