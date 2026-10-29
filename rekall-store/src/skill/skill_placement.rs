//! The group a person put one of their own Claude skills under, kept in Rekall alone: the skill's
//! file never moves. The skill is named by its path under `~/.claude`.

use rekall_common::{Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const SKILL_PATH_MAX: usize = 300;
pub const GROUP_LABEL_MAX: usize = 60;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "skill_placement")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub skill_path: String,
    pub group_label: String,
    pub placed_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("SkillPlacement")
            .not_blank("skillPath", Some(&self.skill_path))
            .max_length("skillPath", Some(&self.skill_path), SKILL_PATH_MAX)
            .not_blank("group", Some(&self.group_label))
            .max_length("group", Some(&self.group_label), GROUP_LABEL_MAX)
            .finish()
    }
}
