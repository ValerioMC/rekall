use std::fmt;

use serde::Serialize;

/// One broken rule: where (`sheets[0].links["#3"]`) and what.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DocumentViolation {
    pub path: String,
    pub message: String,
}

impl DocumentViolation {
    pub fn new(path: &str, message: &str) -> Self {
        Self { path: path.to_string(), message: message.to_string() }
    }
}

impl fmt::Display for DocumentViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}
