//! A stored Semantic Graph: what a session was asked, and the graph it answered with, kept as
//! the JSON text `rekall-diagram` reads. It belongs to a project, whose folder its source paths
//! are relative to, and remembers the task it was generated from while that task exists.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::{Phase, Violations};

pub const TITLE_MAX_CHARACTERS: usize = 200;
pub const QUESTION_MAX_CHARACTERS: usize = 4000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "diagram")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub project_id: Id,
    pub task_id: Option<Id>,
    pub title: String,
    /// The request or context the graph answers, in the words it was asked in.
    pub question: String,
    pub graph_json: String,
    pub node_count: i32,
    pub edge_count: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::project::Entity", from = "Column::ProjectId", to = "super::project::Column::Id", on_delete = "Cascade")]
    Project,
    #[sea_orm(belongs_to = "super::task::Entity", from = "Column::TaskId", to = "super::task::Column::Id", on_delete = "SetNull")]
    Task,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self, phase: Phase) -> Result<(), RekallError> {
        Violations::new("Diagram", phase)
            .not_blank("title", Some(&self.title))
            .size("title", Some(&self.title), TITLE_MAX_CHARACTERS)
            .size("question", Some(&self.question), QUESTION_MAX_CHARACTERS)
            .not_blank("graphJson", Some(&self.graph_json))
            .finish()
    }
}
