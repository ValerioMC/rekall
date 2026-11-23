//! What the person added to a decision after it was answered, dated.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const TEXT_MAX_CHARACTERS: usize = 2_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_decision_detail")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub decision_id: Id,
    pub text: String,
    pub added_at: Instant,
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
    pub fn new(decision_id: Id, text: String, added_at: Instant) -> Self {
        Self { id: Id::random(), decision_id, text, added_at }
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("Decision detail")
            .not_blank("text", Some(&self.text))
            .max_length("text", Some(&self.text), TEXT_MAX_CHARACTERS)
            .finish()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/decision/entity/task_decision_detail_tests.rs"]
mod tests;
