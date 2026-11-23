//! One line of the conversation a decision came from.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::decision::enums::Speaker;

pub const TEXT_MAX_CHARACTERS: usize = 4_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_decision_line")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub decision_id: Id,
    pub position: i32,
    pub speaker: Speaker,
    pub text: String,
    pub spoken_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::task_decision::Entity", from = "Column::DecisionId", to = "crate::task_decision::Column::Id", on_delete = "Cascade")]
    Decision,
}

impl Related<crate::task_decision::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Decision.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new(decision_id: Id, position: i32, speaker: Speaker, text: String, spoken_at: Instant) -> Self {
        Self { id: Id::random(), decision_id, position, speaker, text, spoken_at }
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("Decision line")
            .not_blank("text", Some(&self.text))
            .max_length("text", Some(&self.text), TEXT_MAX_CHARACTERS)
            .finish()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/decision/entity/task_decision_line_tests.rs"]
mod tests;
