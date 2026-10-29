//! A route between two steps of one deployment graph, by outcome, or from a step to the end. It
//! becomes a route between the task's steps when the graph is attached.

use rekall_common::{Id, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::step_link::OUTCOME_MAX_CHARACTERS;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "graph_route")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub graph_id: Id,
    pub from_step_id: Id,
    /// The step the route leads to; `None` ends the flow there.
    pub to_step_id: Option<Id>,
    pub outcome: String,
    pub position: i32,
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
        Violations::new("GraphRoute")
            .not_blank("outcome", Some(&self.outcome))
            .slug("outcome", Some(&self.outcome))
            .max_length("outcome", Some(&self.outcome), OUTCOME_MAX_CHARACTERS)
            .finish()
    }
}
