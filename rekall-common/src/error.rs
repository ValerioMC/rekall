//! The error vocabulary shared by every layer. Each transport maps a variant to its own status:
//! the REST API sends `NotFound`/`UnknownAnchor` as 404, `Conflict`/`AmbiguousAnchor`/`Integrity`
//! as 409, `IllegalArgument` as 400 and `Internal` as 500; the MCP server to its error codes.

use std::fmt;

pub type Result<T, E = RekallError> = std::result::Result<T, E>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RekallError {
    /// A record addressed by id is not there.
    NotFound(String),
    /// The request is well formed but the state refuses it.
    Conflict(String),
    /// The caller's input breaks a rule.
    IllegalArgument(String),
    /// An anchor or id that resolves to nothing.
    UnknownAnchor(String),
    /// A bare term that resolves to more than one record.
    AmbiguousAnchor { message: String, candidates: Vec<String> },
    /// The database refused a write. The text is the most specific cause, with the constraint's
    /// name when it is one `NAMED_CONSTRAINTS` knows.
    Integrity(String),
    /// Anything unexpected; the text is `kind: message`.
    Internal(String),
}

/// What went wrong inside Rekall when a failure is not the caller's: the prefix of an `Internal`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InternalFailure {
    Database,
    Io,
    /// Stored rows contradict a rule the schema does not enforce, such as a task with no project.
    Integrity,
    /// A blocking job handed to Tokio panicked or was cancelled.
    Background,
    Encoding,
    /// A stored value no longer parses, such as a diagram's graph.
    CorruptData,
}

impl InternalFailure {
    fn label(self) -> &'static str {
        match self {
            Self::Database => "Database",
            Self::Io => "I/O",
            Self::Integrity => "Integrity",
            Self::Background => "Background job",
            Self::Encoding => "Encoding",
            Self::CorruptData => "Corrupt data",
        }
    }
}

impl RekallError {
    /// "No {what} with id {id}".
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

    pub fn ambiguous(value: &str, candidates: Vec<String>) -> Self {
        Self::AmbiguousAnchor {
            message: format!("'{}' matches {} records: {}", value, candidates.len(), candidates.join(", ")),
            candidates,
        }
    }

    pub fn internal(kind: InternalFailure, message: impl fmt::Display) -> Self {
        Self::Internal(format!("{}: {message}", kind.label()))
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

/// The constraints the REST layer explains in words. SQLite names the columns of a broken
/// constraint, so each is mapped back to the constraint's name here.
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
        Self::internal(InternalFailure::Database, text)
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
#[path = "../tests/unit/error_tests.rs"]
mod tests;
