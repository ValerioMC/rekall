use std::fmt;

use super::DocumentViolation;

/// The most violations one report lists; past it, fixing the first ones is the next step anyway.
pub const MAX_REPORTED: usize = 50;

/// Every rule a document broke, in document order. Displayed as one line per violation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentViolations(pub Vec<DocumentViolation>);

impl DocumentViolations {
    /// `Ok` when nothing was found, otherwise the first [`MAX_REPORTED`] of what was.
    pub fn into_result(mut found: Vec<DocumentViolation>) -> Result<(), DocumentViolations> {
        if found.is_empty() {
            return Ok(());
        }
        found.truncate(MAX_REPORTED);
        Err(DocumentViolations(found))
    }
}

impl fmt::Display for DocumentViolations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(f, "The diagram breaks {} rule(s):\n{}", self.0.len(), lines.join("\n"))
    }
}

impl std::error::Error for DocumentViolations {}
