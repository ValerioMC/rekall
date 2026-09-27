use std::fmt;

use serde::Serialize;

/// One broken rule: where (`nodes[3].title`) and what.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GraphViolation {
    pub path: String,
    pub message: String,
}

impl fmt::Display for GraphViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}
