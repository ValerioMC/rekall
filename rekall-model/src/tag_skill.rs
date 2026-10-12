//! A Claude skill every task carrying the tag uses by default. Resolved when a task is read, never
//! copied onto the task, so editing a tag changes every task under it.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::skill_name;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tag_skill")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub tag_id: Id,
    pub skill_name: String,
    pub position: i32,
    pub created_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tag::Entity",
        from = "Column::TagId",
        to = "super::tag::Column::Id",
        on_delete = "Cascade"
    )]
    Tag,
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        skill_name::validate(&self.skill_name)
    }
}
