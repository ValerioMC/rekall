use std::fmt;

use serde::{Deserialize, Serialize};

/// What an edge says about its two ends. Unknown snake_case names are kept as `Custom`, for the
/// same reason as [`crate::NodeKind`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum RelationKind {
    Calls,
    Contains,
    DependsOn,
    TransitionsTo,
    Triggers,
    Reads,
    Writes,
    Produces,
    Consumes,
    LeadsTo,
    ConditionallyLeadsTo,
    Custom(String),
}

impl RelationKind {
    pub const KNOWN: [RelationKind; 11] = [
        RelationKind::Calls,
        RelationKind::Contains,
        RelationKind::DependsOn,
        RelationKind::TransitionsTo,
        RelationKind::Triggers,
        RelationKind::Reads,
        RelationKind::Writes,
        RelationKind::Produces,
        RelationKind::Consumes,
        RelationKind::LeadsTo,
        RelationKind::ConditionallyLeadsTo,
    ];

    pub fn name(&self) -> &str {
        match self {
            RelationKind::Calls => "calls",
            RelationKind::Contains => "contains",
            RelationKind::DependsOn => "depends_on",
            RelationKind::TransitionsTo => "transitions_to",
            RelationKind::Triggers => "triggers",
            RelationKind::Reads => "reads",
            RelationKind::Writes => "writes",
            RelationKind::Produces => "produces",
            RelationKind::Consumes => "consumes",
            RelationKind::LeadsTo => "leads_to",
            RelationKind::ConditionallyLeadsTo => "conditionally_leads_to",
            RelationKind::Custom(name) => name,
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, RelationKind::Custom(_))
    }
}

impl From<String> for RelationKind {
    fn from(name: String) -> Self {
        RelationKind::KNOWN.into_iter().find(|kind| kind.name() == name).unwrap_or(RelationKind::Custom(name))
    }
}

impl From<RelationKind> for String {
    fn from(kind: RelationKind) -> Self {
        kind.name().to_string()
    }
}

impl fmt::Display for RelationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
