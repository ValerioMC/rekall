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
mod tests {
    use super::*;

    fn new_task() -> Model {
        Model::new("in-app-claude".into(), "In-app Claude execution".into(), Id::random(), Instant::now())
    }

    #[test]
    fn a_fresh_task_opens_open_review_active_with_nothing_behind_it() {
        let task = new_task();
        assert_eq!(task.review_state, TaskStepState::Open);
        assert!(review_active(&[]));
        assert!(task.claimed_at.is_none());
        assert!(task.accepted_at.is_none());
        assert!(task.review_note.is_none());
    }

    #[test]
    fn running_then_claimed_then_accepted_each_move_stamps_its_own_moment() {
        let mut task = new_task();
        task.mark_review_state(TaskStepState::Running);
        assert!(task.claimed_at.is_none());
        assert!(task.accepted_at.is_none());

        task.mark_review_state(TaskStepState::Claimed);
        assert!(task.claimed_at.is_some());
        assert!(task.accepted_at.is_none());

        task.mark_review_state(TaskStepState::Done);
        assert!(task.accepted_at.is_some());
        assert!(task.claimed_at.is_some(), "the claim moment is kept when the work is later accepted");
    }

    #[test]
    fn accepting_straight_from_open_still_records_that_it_must_have_been_claimed() {
        let mut task = new_task();
        task.mark_review_state(TaskStepState::Done);
        assert!(task.claimed_at.is_some());
        assert!(task.accepted_at.is_some());
    }

    #[test]
    fn a_send_back_note_is_kept_while_open_and_dropped_the_moment_it_is_claimed_again() {
        let mut task = new_task();
        task.mark_review_state(TaskStepState::Open);
        task.review_note = Some("the export column is still wrong".into());
        assert_eq!(task.review_note.as_deref(), Some("the export column is still wrong"));

        task.mark_review_state(TaskStepState::Claimed);
        assert!(task.review_note.is_none());
    }

    #[test]
    fn sending_back_clears_every_moment() {
        let mut task = new_task();
        task.mark_review_state(TaskStepState::Running);
        task.mark_review_state(TaskStepState::Claimed);
        task.mark_review_state(TaskStepState::Done);

        task.mark_review_state(TaskStepState::Open);

        assert_eq!(task.review_state, TaskStepState::Open);
        assert!(task.claimed_at.is_none());
        assert!(task.accepted_at.is_none());
    }

    #[test]
    fn marking_the_state_it_is_already_in_changes_nothing() {
        let mut task = new_task();
        task.mark_review_state(TaskStepState::Claimed);
        let claimed_at = task.claimed_at;
        std::thread::sleep(std::time::Duration::from_millis(2));
        task.mark_review_state(TaskStepState::Claimed);
        assert_eq!(task.claimed_at, claimed_at);
    }

    #[test]
    fn only_drafts_count_as_no_checklist() {
        assert!(review_active(&[TaskStepState::Draft, TaskStepState::Draft]));
        assert!(!review_active(&[TaskStepState::Draft, TaskStepState::Open]));
    }
}
