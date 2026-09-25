//! A configured badge a task can carry: a name, a glowing icon key and a glow colour key, both
//! chosen from the fixed sets the console offers.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::{Phase, Violations};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::task::Entity")]
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
        Violations::new("Tag", phase)
            .not_blank("name", Some(&self.name))
            .size("name", Some(&self.name), 60)
            .not_blank("icon", Some(&self.icon))
            .size("icon", Some(&self.icon), 40)
            .not_blank("color", Some(&self.color))
            .size("color", Some(&self.color), 40)
            .finish()
    }
}
