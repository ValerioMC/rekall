//! A unit of work inside a project. `label` is unique per project and is the anchor; `title` is
//! free text. A task with no checklist (or only draft steps) carries its own review line in
//! `review_state`, `claimed_at`, `accepted_at` and `review_note`.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::{Phase, Violations};
use crate::enums::{TaskStatus, TaskStepState};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub label: String,
    pub title: String,
    pub status: TaskStatus,
    pub description: Option<String>,
    pub review_state: TaskStepState,
    pub claimed_at: Option<Instant>,
    pub accepted_at: Option<Instant>,
    pub review_note: Option<String>,
    pub project_id: Id,
    pub tag_id: Option<Id>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id",
        on_delete = "Cascade"
    )]
    Project,
    #[sea_orm(
        belongs_to = "super::tag::Entity",
        from = "Column::TagId",
        to = "super::tag::Column::Id",
        on_delete = "SetNull"
    )]
    Tag,
    #[sea_orm(has_many = "super::task_step::Entity")]
    TaskStep,
    #[sea_orm(has_one = "super::wrapup::Entity")]
    Wrapup,
    #[sea_orm(has_many = "super::document_task::Entity")]
    DocumentTask,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tag.def()
    }
}

impl Related<super::task_step::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskStep.def()
    }
}

impl Related<super::wrapup::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Wrapup.def()
    }
}

impl Related<super::document::Entity> for Entity {
    fn to() -> RelationDef {
        super::document_task::Relation::Document.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::document_task::Relation::Task.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// `Task.reviewActive()`: true while the task has no checklist, only draft steps counting as
/// none. Once a step is past `DRAFT` the checklist is the source of truth and the review columns
/// are ignored.
pub fn review_active<'a>(step_states: impl IntoIterator<Item = &'a TaskStepState>) -> bool {
    step_states.into_iter().all(|state| *state == TaskStepState::Draft)
}

impl Model {
    /// A new task as `new Task(label, title)` left it: `TODO`, review line `OPEN`, no moments.
    pub fn new(label: String, title: String, project_id: Id, now: Instant) -> Self {
        Self {
            id: Id::random(),
            label,
            title,
            status: TaskStatus::Todo,
            description: None,
            review_state: TaskStepState::Open,
            claimed_at: None,
            accepted_at: None,
            review_note: None,
            project_id,
            tag_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// `Task.markReviewState`: move the task-scoped review line. `OPEN` before a run or after a
    /// send back, `RUNNING` during a session, `CLAIMED` once a wrapup lands, `DONE` on accept.
    /// Each move resets the claim and accept moments and the note to match; marking the state
    /// the task is already in changes nothing.
    pub fn mark_review_state(&mut self, next: TaskStepState) {
        if self.review_state == next {
            return;
        }
        self.review_state = next;
        let now = Instant::now();
        match next {
            TaskStepState::Open | TaskStepState::Running => {
                self.claimed_at = None;
                self.accepted_at = None;
                self.review_note = None;
            }
            TaskStepState::Claimed => {
                self.claimed_at = Some(now);
                self.accepted_at = None;
                self.review_note = None;
            }
            TaskStepState::Done => {
                if self.claimed_at.is_none() {
                    self.claimed_at = Some(now);
                }
                self.accepted_at = Some(now);
                self.review_note = None;
            }
            // DRAFT is a step's state, never the task's; the switch in Java had no case for it.
            TaskStepState::Draft => {}
        }
    }

    pub fn anchor(&self, project_label: &str) -> String {
        format!("project:{project_label} task:{}", self.label)
    }

    pub fn validate(&self, phase: Phase) -> Result<(), RekallError> {
        Violations::new("Task", phase)
            .not_blank("label", Some(&self.label))
            .size("label", Some(&self.label), 160)
            .slug("label", Some(&self.label))
            .not_blank("title", Some(&self.title))
            .size("title", Some(&self.title), 200)
            .size("reviewNote", self.review_note.as_deref(), 2_000)
            .column("description", self.description.as_deref(), 100_000)
            .finish()
    }
}

#[cfg(test)]
#[path = "task_tests.rs"]
mod tests;
