//! One answer a session offered for a decision, with what choosing it would cause.

use rekall_kernel::{Id, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const LABEL_MAX_CHARACTERS: usize = 200;
pub const EFFECT_MAX_CHARACTERS: usize = 1_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_decision_option")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub decision_id: Id,
    pub position: i32,
    pub label: String,
    pub effect: Option<String>,
    pub recommended: bool,
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
    pub fn new(decision_id: Id, position: i32, label: String) -> Self {
        Self { id: Id::random(), decision_id, position, label, effect: None, recommended: false }
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("Decision option")
            .not_blank("label", Some(&self.label))
            .max_length("label", Some(&self.label), LABEL_MAX_CHARACTERS)
            .max_length("effect", self.effect.as_deref(), EFFECT_MAX_CHARACTERS)
            .finish()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/decision/entity/task_decision_option_tests.rs"]
mod tests;
