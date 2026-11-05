//! One message of a task's thread. A `REVISION` keeps the description before (`quoted_markdown`) and
//! after (`body_markdown`) a rewrite; the other kinds carry only the body. It stays `PENDING` until
//! Run revision takes it into a numbered `pass` (`RUNNING`), and `APPLIED` once a session answers it
//! with a reply. `follows_seq` names the earlier entry it adds detail to. `seq` numbers the entries
//! of one task from 1.

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
    pub pass: Option<i32>,
    pub taken_at: Option<Instant>,
    pub follows_seq: Option<i32>,
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
    pub fn new(task_id: Id, seq: i32, kind: FeedbackKind, quoted_markdown: Option<String>, body_markdown: String, follows_seq: Option<i32>, now: Instant) -> Self {
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
            pass: None,
            taken_at: None,
            follows_seq,
        }
    }

    pub fn pending(&self) -> bool {
        self.state == FeedbackState::Pending
    }

    /// Waiting for a session or taken into a pass: what a reply may still answer.
    pub fn open(&self) -> bool {
        matches!(self.state, FeedbackState::Pending | FeedbackState::Running)
    }

    pub fn take(&mut self, pass: i32, now: Instant) {
        self.state = FeedbackState::Running;
        self.pass = Some(pass);
        self.taken_at = Some(now);
        self.updated_at = now;
    }

    pub fn release(&mut self, now: Instant) {
        self.state = FeedbackState::Pending;
        self.pass = None;
        self.taken_at = None;
        self.updated_at = now;
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
            .require(self.follows_seq.map_or(true, |followed| followed >= 1 && followed != self.seq), "followsSeq names an earlier entry, never the entry itself")
            .require(!(self.kind.is_revision() && self.follows_seq.is_some()), "a revision follows the description, not an entry")
            .finish()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/feedback/entity/task_feedback_tests.rs"]
mod tests;
