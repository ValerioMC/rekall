//! What a task's implementation looks like now. One per task (`task_id` is unique), replaced
//! whole, capped at 20,000 characters.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::{Phase, Violations};
use crate::enums::WrapupAuthor;

pub const MAX_CHARACTERS: usize = 20_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "wrapup")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    #[sea_orm(unique)]
    pub task_id: Id,
    pub body_markdown: String,
    pub written_by: WrapupAuthor,
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
    pub fn validate(&self, phase: Phase) -> Result<(), RekallError> {
        Violations::new("Wrapup", phase)
            .not_blank("bodyMarkdown", Some(&self.body_markdown))
            .size("bodyMarkdown", Some(&self.body_markdown), MAX_CHARACTERS)
            .finish()
    }
}
