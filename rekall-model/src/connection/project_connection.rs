//! A project's use of one of its company's connections. The target narrows it to this project:
//! the GitLab project path, the Kubernetes namespace its work goes to, or the Jira project key.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const TARGET_MAX_CHARACTERS: usize = 500;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "project_connection")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub project_id: Id,
    pub connection_id: Id,
    pub target: Option<String>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::project::Entity", from = "Column::ProjectId", to = "crate::project::Column::Id", on_delete = "Cascade")]
    Project,
    #[sea_orm(
        belongs_to = "crate::external_connection::Entity",
        from = "Column::ConnectionId",
        to = "crate::external_connection::Column::Id",
        on_delete = "Cascade"
    )]
    Connection,
}

impl Related<crate::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<crate::external_connection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Connection.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new(project_id: Id, connection_id: Id, target: Option<String>, now: Instant) -> Self {
        Self { id: Id::random(), project_id, connection_id, target, created_at: now, updated_at: now }
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("ProjectConnection").max_length("target", self.target.as_deref(), TARGET_MAX_CHARACTERS).finish()
    }
}
