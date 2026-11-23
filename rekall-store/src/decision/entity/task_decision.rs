//! One question about a task and what became of it. `step_number` is empty when the decision is
//! about the task as a whole; `key` is the slug a session addresses the record by, unique per task.
//! A changed answer keeps the one it replaced in `previous_answer`.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::decision::enums::{DecisionSource, DecisionState};

pub const KEY_MAX_CHARACTERS: usize = 100;
pub const TOPIC_MAX_CHARACTERS: usize = 200;
pub const QUESTION_MAX_CHARACTERS: usize = 500;
pub const WHY_MAX_CHARACTERS: usize = 2_000;
pub const ANSWER_MAX_CHARACTERS: usize = 4_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_decision")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub step_number: Option<i32>,
    pub key: String,
    pub topic: Option<String>,
    pub question: String,
    pub why: Option<String>,
    pub state: DecisionState,
    pub answer: Option<String>,
    pub previous_answer: Option<String>,
    pub changed_at: Option<Instant>,
    pub source: DecisionSource,
    pub created_at: Instant,
    pub updated_at: Instant,
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
    pub fn new(task_id: Id, key: String, question: String, source: DecisionSource, now: Instant) -> Self {
        Self {
            id: Id::random(),
            task_id,
            step_number: None,
            key,
            topic: None,
            question,
            why: None,
            state: DecisionState::Open,
            answer: None,
            previous_answer: None,
            changed_at: None,
            source,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn change_answer(&mut self, answer: String, now: Instant) {
        self.previous_answer = self.answer.take();
        self.answer = Some(answer);
        self.changed_at = Some(now);
        self.updated_at = now;
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("Decision")
            .in_range("stepNumber", self.step_number.unwrap_or(1), 1, i32::MAX)
            .not_blank("key", Some(&self.key))
            .slug("key", Some(&self.key))
            .max_length("key", Some(&self.key), KEY_MAX_CHARACTERS)
            .max_length("topic", self.topic.as_deref(), TOPIC_MAX_CHARACTERS)
            .not_blank("question", Some(&self.question))
            .max_length("question", Some(&self.question), QUESTION_MAX_CHARACTERS)
            .max_length("why", self.why.as_deref(), WHY_MAX_CHARACTERS)
            .max_length("answer", self.answer.as_deref(), ANSWER_MAX_CHARACTERS)
            .max_length("previousAnswer", self.previous_answer.as_deref(), ANSWER_MAX_CHARACTERS)
            .finish()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/decision/entity/task_decision_tests.rs"]
mod tests;
