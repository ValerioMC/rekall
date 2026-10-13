//! A route between two steps of one task: when the `from` step ends with `outcome`, the work
//! moves to the `to` step. A task whose steps carry routes is a flow, not a list.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::slug::Slug;

pub const OUTCOME_MAX_CHARACTERS: usize = 60;

/// Routes leaving one step: past this the step is deciding too much and wants splitting.
pub const ROUTES_PER_STEP_MAX: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "step_link")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub from_step_id: Id,
    pub to_step_id: Id,
    /// A slug, so a session names it the way the console shows it: `app-error`.
    pub outcome: String,
    pub position: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::TaskId",
        to = "super::task::Column::Id",
        on_delete = "Cascade"
    )]
    Task,
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new(task_id: Id, from_step_id: Id, to_step_id: Id, outcome: String, position: i32, now: Instant) -> Self {
        Self { id: Id::random(), task_id, from_step_id, to_step_id, outcome, position, created_at: now, updated_at: now }
    }

    /// A step that repeats is reached again through another step; a route onto itself would
    /// loop without anything in between deciding to.
    pub fn validate(&self) -> Result<(), RekallError> {
        if self.from_step_id == self.to_step_id {
            return Err(RekallError::illegal("A route has to lead to another step. A step cannot route to itself."));
        }
        Violations::new("StepLink")
            .not_blank("outcome", Some(&self.outcome))
            .slug("outcome", Some(&self.outcome))
            .max_length("outcome", Some(&self.outcome), OUTCOME_MAX_CHARACTERS)
            .finish()
    }
}

/// An outcome as typed, normalised the way a label is: `App error` becomes `app-error`.
pub fn normalise_outcome(raw: Option<&str>) -> Result<String, RekallError> {
    Slug::of(raw).map_err(|_| {
        RekallError::illegal(format!(
            "'{}' leaves no usable outcome. Name what happened: `passed`, `app-error`, `pipeline-error`.",
            raw.unwrap_or("")
        ))
    })
}

#[cfg(test)]
#[path = "../tests/unit/step_link_tests.rs"]
mod tests;
