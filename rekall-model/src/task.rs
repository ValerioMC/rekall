//! A unit of work inside a project. `label` is unique per project and is the anchor; `title` is
//! free text. A task with no checklist (or only draft steps) carries its own review line in
//! `review_state`, `claimed_at`, `accepted_at` and `review_note`. An epic (`kind`) holds no steps
//! and gathers the tasks of its project whose `epic_id` names it.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::enums::{TaskKind, TaskStatus, TaskStepState};

pub const TITLE_MAX_CHARACTERS: usize = 200;

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
    /// When a deployment graph was laid on the task; a task is laid once.
    pub graph_laid_at: Option<Instant>,
    /// The project's release graph is not laid when the console runs this task.
    pub release_skipped: bool,
    pub kind: TaskKind,
    pub epic_id: Option<Id>,
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

/// True while the task has no checklist, only draft steps counting as
/// none. Once a step is past `DRAFT` the checklist is the source of truth and the review columns
/// are ignored.
pub fn review_active<'a>(step_states: impl IntoIterator<Item = &'a TaskStepState>) -> bool {
    step_states.into_iter().all(|state| *state == TaskStepState::Draft)
}

impl Model {
    /// A new task: `TODO`, review line `OPEN`, no moments.
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
            graph_laid_at: None,
            release_skipped: false,
            kind: TaskKind::Task,
            epic_id: None,
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
            // DRAFT is a step's state, never the task's.
            TaskStepState::Draft => {}
        }
    }

    pub fn is_epic(&self) -> bool {
        self.kind == TaskKind::Epic
    }

    /// An epic gathers tasks and is never worked itself; `refused` says what it was asked to do.
    pub fn require_workable(&self, refused: &str) -> Result<(), RekallError> {
        if self.is_epic() {
            return Err(RekallError::illegal(format!(
                "'{}' is an epic, and an epic {refused}. Work one of the tasks that belong to it instead.",
                self.label
            )));
        }
        Ok(())
    }

    /// An epic gathers plain tasks of its own project, one level deep: an epic never sits under
    /// another, and a task is never its own epic.
    pub fn require_joinable(&self, epic: &Model) -> Result<(), RekallError> {
        if !epic.is_epic() {
            return Err(RekallError::illegal(format!("'{}' is not an epic, so no task can belong to it.", epic.label)));
        }
        if self.is_epic() {
            return Err(RekallError::illegal("An epic cannot belong to another epic."));
        }
        if epic.id == self.id {
            return Err(RekallError::illegal("A task cannot be its own epic."));
        }
        if epic.project_id != self.project_id {
            return Err(RekallError::illegal(format!(
                "The epic '{}' is in another project. A task belongs to an epic of its own project.",
                epic.label
            )));
        }
        Ok(())
    }

    pub fn anchor(&self, project_label: &str) -> String {
        format!("project:{project_label} task:{}", self.label)
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("Task")
            .not_blank("label", Some(&self.label))
            .max_length("label", Some(&self.label), 160)
            .slug("label", Some(&self.label))
            .not_blank("title", Some(&self.title))
            .max_length("title", Some(&self.title), TITLE_MAX_CHARACTERS)
            .max_length("reviewNote", self.review_note.as_deref(), 2_000)
            .max_length("description", self.description.as_deref(), 100_000)
            .finish()
    }
}

#[cfg(test)]
#[path = "../tests/unit/task_tests.rs"]
mod tests;
