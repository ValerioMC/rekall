use std::fmt;

use super::GraphViolation;

/// Every rule a graph broke, in document order. Displayed as one line per violation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphViolations(pub Vec<GraphViolation>);

impl fmt::Display for GraphViolations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(f, "The graph breaks {} rule(s):\n{}", self.0.len(), lines.join("\n"))
    }
}

impl std::error::Error for GraphViolations {}
