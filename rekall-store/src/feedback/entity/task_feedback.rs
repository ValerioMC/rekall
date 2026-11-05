//! One message of a task's thread. A `REVISION` keeps the description before (`quoted_markdown`) and
//! after (`body_markdown`) a rewrite; the other kinds carry only the body. It stays `PENDING` until a
//! session answers it with a reply, and `seq` numbers the entries of one task from 1.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::enums::{FeedbackKind, FeedbackState};

pub const BODY_MAX_CHARACTERS: usize = 100_000;
pub const REPLY_MAX_CHARACTERS: usize = 2_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_feedback")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub seq: i32,
    pub kind: FeedbackKind,
    pub state: FeedbackState,
    pub quoted_markdown: Option<String>,
    pub body_markdown: String,
    pub reply_markdown: Option<String>,
    pub created_at: Instant,
    pub updated_at: Instant,
    pub applied_at: Option<Instant>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::task::Entity", from = "Column::TaskId", to = "crate::task::Column::Id", on_delete = "Cascade")]
    Task,
}

impl Related<crate::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new(task_id: Id, seq: i32, kind: FeedbackKind, quoted_markdown: Option<String>, body_markdown: String, now: Instant) -> Self {
        Self {
            id: Id::random(),
            task_id,
            seq,
            kind,
            state: FeedbackState::Pending,
            quoted_markdown,
            body_markdown,
            reply_markdown: None,
            created_at: now,
            updated_at: now,
            applied_at: None,
        }
    }

    pub fn pending(&self) -> bool {
        self.state == FeedbackState::Pending
    }

    pub fn mark_applied(&mut self, reply_markdown: String, now: Instant) {
        self.state = FeedbackState::Applied;
        self.reply_markdown = Some(reply_markdown);
        self.applied_at = Some(now);
        self.updated_at = now;
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("Feedback")
            .in_range("seq", self.seq, 1, i32::MAX)
            .not_blank("bodyMarkdown", Some(&self.body_markdown))
            .max_length("bodyMarkdown", Some(&self.body_markdown), BODY_MAX_CHARACTERS)
            .max_length("quotedMarkdown", self.quoted_markdown.as_deref(), BODY_MAX_CHARACTERS)
            .max_length("replyMarkdown", self.reply_markdown.as_deref(), REPLY_MAX_CHARACTERS)
            .require(self.kind.is_revision() == self.quoted_markdown.is_some(), "only a revision carries quotedMarkdown, and a revision must")
            .finish()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/feedback/entity/task_feedback_tests.rs"]
mod tests;
