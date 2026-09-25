//! The one run queue: when it starts, the usage ceiling it stops at, and how the sessions it
//! opens are launched. There is a single row; the queued tasks are run queue items.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::{Phase, Violations};
use crate::enums::RunQueueState;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "run_queue")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub state: RunQueueState,
    /// When a scheduled queue starts; null for one started at once or not started at all.
    pub start_at: Option<Instant>,
    /// The usage percentage at which no new task or step is started; null means no ceiling.
    pub ceiling_percent: Option<i32>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    /// While `HOLDING`: when the queue looks at usage again.
    pub hold_until: Option<Instant>,
    pub hold_reason: Option<String>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// `new RunQueue()`: idle, no ceiling, nothing scheduled.
    pub fn new(now: Instant) -> Self {
        Self {
            id: Id::random(),
            state: RunQueueState::Idle,
            start_at: None,
            ceiling_percent: None,
            skip_permissions: false,
            model: None,
            effort: None,
            hold_until: None,
            hold_reason: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// `RunQueue.moveTo`: move to `next`, clearing whatever belonged only to the state being left.
    pub fn move_to(&mut self, next: RunQueueState) {
        self.state = next;
        if next != RunQueueState::Scheduled {
            self.start_at = None;
        }
        if next != RunQueueState::Holding {
            self.hold_until = None;
            self.hold_reason = None;
        }
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("RunQueue", Phase::Update)
            .column("model", self.model.as_deref(), 20)
            .column("effort", self.effort.as_deref(), 20)
            .column("hold_reason", self.hold_reason.as_deref(), 500)
            .finish()
    }
}
