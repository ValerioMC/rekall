//! The error vocabulary shared by every layer.
//!
//! The Java code expressed these as exception classes, and each transport mapped the class to a
//! status: `NotFoundException` and `UnknownAnchorException` to 404, `ConflictException` and
//! `AmbiguousAnchorException` to 409, `IllegalArgumentException` to 400, a database constraint to
//! 409 with an explanation, anything else to 500. One enum carries the same distinctions, so the
//! REST layer and the MCP layer can map them exactly as before.

use std::fmt;

pub type Result<T, E = RekallError> = std::result::Result<T, E>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RekallError {
    /// `NotFoundException`: a record addressed by id is not there.
    NotFound(String),
    /// `ConflictException`: the request is well formed but the state refuses it.
    Conflict(String),
    /// `IllegalArgumentException`: the caller's input breaks a rule.
    IllegalArgument(String),
    /// `UnknownAnchorException`: an anchor or id that resolves to nothing.
    UnknownAnchor(String),
    /// `AmbiguousAnchorException`: a bare term that resolves to more than one record.
    AmbiguousAnchor { message: String, candidates: Vec<String> },
    /// `DataIntegrityViolationException`: the database refused a write. The text is the most
    /// specific cause, with the constraint's name in it the way H2 reported it.
    Integrity(String),
    /// Anything unexpected; the text is `SimpleClassName: message`, as the Java handler printed.
    Internal(String),
}

impl RekallError {
    /// `new NotFoundException(what, id)`: "No %s with id %s".
    pub fn not_found(what: &str, id: impl fmt::Display) -> Self {
        Self::NotFound(format!("No {what} with id {id}"))
    }

    pub fn not_found_msg(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn illegal(message: impl Into<String>) -> Self {
        Self::IllegalArgument(message.into())
    }

    pub fn unknown_anchor(message: impl Into<String>) -> Self {
        Self::UnknownAnchor(message.into())
    }

    /// `new AmbiguousAnchorException(value, candidates)`.
    pub fn ambiguous(value: &str, candidates: Vec<String>) -> Self {
        Self::AmbiguousAnchor {
            message: format!("'{}' matches {} records: {}", value, candidates.len(), candidates.join(", ")),
            candidates,
        }
    }

    pub fn internal(kind: &str, message: impl fmt::Display) -> Self {
        Self::Internal(format!("{kind}: {message}"))
    }

    pub fn message(&self) -> &str {
        match self {
            Self::NotFound(m)
            | Self::Conflict(m)
            | Self::IllegalArgument(m)
            | Self::UnknownAnchor(m)
            | Self::Integrity(m)
            | Self::Internal(m) => m,
            Self::AmbiguousAnchor { message, .. } => message,
        }
    }

    pub fn is_illegal_argument(&self) -> bool {
        matches!(self, Self::IllegalArgument(_))
    }
}

impl fmt::Display for RekallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for RekallError {}

/// The constraints the REST layer explains in words. H2 named them in its message and the Java
/// handler matched on that name; SQLite names the columns instead, so the name is put back here.
const NAMED_CONSTRAINTS: &[(&str, &str)] = &[
    ("project.company_id, project.label", "UQ_PROJECT_COMPANY_LABEL"),
    ("task.project_id, task.label", "UQ_TASK_PROJECT_LABEL"),
    ("company.name", "UQ_COMPANY_NAME"),
    ("tag.name", "UQ_TAG_NAME"),
    ("wrapup.task_id", "UQ_WRAPUP_TASK"),
];

impl From<sea_orm::DbErr> for RekallError {
    fn from(error: sea_orm::DbErr) -> Self {
        let text = error.to_string();
        if let Some(violation) = integrity_cause(&text) {
            return Self::Integrity(violation);
        }
        Self::internal("DataAccessException", text)
    }
}

fn integrity_cause(text: &str) -> Option<String> {
    let at = text.find("UNIQUE constraint failed").or_else(|| text.find("FOREIGN KEY constraint failed"))
        .or_else(|| text.find("NOT NULL constraint failed"))
        .or_else(|| text.find("CHECK constraint failed"))?;
    let cause = text[at..].trim_end().to_string();
    for (columns, name) in NAMED_CONSTRAINTS {
        if cause.contains(columns) {
            return Some(format!("{cause} ({name})"));
        }
    }
    Some(cause)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ambiguous_anchor_lists_its_candidates() {
        let error = RekallError::ambiguous("setup", vec!["project:a".into(), "project:b".into()]);
        assert_eq!(error.message(), "'setup' matches 2 records: project:a, project:b");
    }

    #[test]
    fn a_unique_violation_carries_the_constraint_name() {
        let cause = integrity_cause("error returned from database: (code: 2067) UNIQUE constraint failed: task.project_id, task.label");
        assert!(cause.unwrap().contains("UQ_TASK_PROJECT_LABEL"));
    }
}
