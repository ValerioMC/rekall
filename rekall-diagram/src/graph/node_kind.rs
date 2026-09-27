use std::fmt;

use serde::{Deserialize, Serialize};

/// What a node stands for. The eight known kinds are what the console draws a shape for; any
/// other snake_case name is kept as `Custom` and drawn generically, so a session can say more
/// than this list does without the format breaking.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum NodeKind {
    Concept,
    Code,
    State,
    Decision,
    Event,
    Data,
    ExternalSystem,
    Action,
    Custom(String),
}

impl NodeKind {
    pub const KNOWN: [NodeKind; 8] = [
        NodeKind::Concept,
        NodeKind::Code,
        NodeKind::State,
        NodeKind::Decision,
        NodeKind::Event,
        NodeKind::Data,
        NodeKind::ExternalSystem,
        NodeKind::Action,
    ];

    pub fn name(&self) -> &str {
        match self {
            NodeKind::Concept => "concept",
            NodeKind::Code => "code",
            NodeKind::State => "state",
            NodeKind::Decision => "decision",
            NodeKind::Event => "event",
            NodeKind::Data => "data",
            NodeKind::ExternalSystem => "external_system",
            NodeKind::Action => "action",
            NodeKind::Custom(name) => name,
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, NodeKind::Custom(_))
    }
}

impl From<String> for NodeKind {
    fn from(name: String) -> Self {
        NodeKind::KNOWN.into_iter().find(|kind| kind.name() == name).unwrap_or(NodeKind::Custom(name))
    }
}

impl From<NodeKind> for String {
    fn from(kind: NodeKind) -> Self {
        kind.name().to_string()
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/graph/node_kind_tests.rs"]
mod tests;
