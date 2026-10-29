//! A graph a project keeps for releasing its work: its steps and routes are rows of their own,
//! written whole with it, and copied onto a task when the graph is attached there.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const TITLE_MAX_CHARACTERS: usize = 200;
pub const DESCRIPTION_MAX_CHARACTERS: usize = 10_000;

/// Steps one graph holds, the same as the drafts a planned flow may leave on a task.
pub const STEPS_MAX: usize = 20;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "deployment_graph")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub project_id: Id,
    pub title: String,
    pub description: String,
    /// Laid on every task of the project the console runs; at most one graph of a project is.
    pub release_default: bool,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::project::Entity", from = "Column::ProjectId", to = "crate::project::Column::Id", on_delete = "Cascade")]
    Project,
}

impl Related<crate::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new(project_id: Id, title: String, description: String, now: Instant) -> Self {
        Self { id: Id::random(), project_id, title, description, release_default: false, created_at: now, updated_at: now }
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("DeploymentGraph")
            .not_blank("title", Some(&self.title))
            .max_length("title", Some(&self.title), TITLE_MAX_CHARACTERS)
            .max_length("description", Some(&self.description), DESCRIPTION_MAX_CHARACTERS)
            .finish()
    }
}
