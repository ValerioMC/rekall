//! One step of a deployment graph: what a session does there and the connection it reaches. It
//! becomes a task's step when the graph is attached, so it keeps a task step's limits.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::task_step::{MAX_CHARACTERS, TITLE_MAX_CHARACTERS};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "graph_step")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub graph_id: Id,
    /// Zero-based; the graph begins at the step in position 0.
    pub position: i32,
    pub title: String,
    pub body_markdown: Option<String>,
    /// The company connection the step reaches; the project has to bind it. `None` reaches nothing.
    pub connection_id: Option<Id>,
    /// How many times a flow may enter the step; `None` sets no limit.
    pub max_visits: Option<i32>,
    /// Where the person dragged the step on the graph's canvas; `None` leaves it to the automatic layout.
    pub flow_x: Option<i32>,
    pub flow_y: Option<i32>,
    pub created_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::deployment_graph::Entity",
        from = "Column::GraphId",
        to = "crate::deployment_graph::Column::Id",
        on_delete = "Cascade"
    )]
    Graph,
}

impl Related<crate::deployment_graph::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Graph.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("GraphStep")
            .not_blank("title", Some(&self.title))
            .max_length("title", Some(&self.title), TITLE_MAX_CHARACTERS)
            .max_length("bodyMarkdown", self.body_markdown.as_deref(), MAX_CHARACTERS)
            .finish()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/graph/graph_step_tests.rs"]
mod tests;
