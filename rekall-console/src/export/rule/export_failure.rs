use std::fmt;

use rekall_kernel::{jstr, InternalFailure, RekallError};
use rekall_store::{ExportFailureKind, ExportFormat};
use rekall_store::project_export::FAILURE_MESSAGE_MAX_CHARACTERS;

use super::export_file_name::noun_of;

/// Where an export stopped and what to tell the person, in plain words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportFailure {
    pub kind: ExportFailureKind,
    pub message: String,
}

impl ExportFailure {
    pub fn rendering(format: ExportFormat, cause: impl fmt::Display) -> Self {
        Self::with(ExportFailureKind::Render, format!("The {} could not be written: {cause}", noun_of(format)))
    }

    pub fn storing(format: ExportFormat, cause: impl fmt::Display) -> Self {
        Self::with(ExportFailureKind::Storage, format!("The {} could not be stored: {cause}", noun_of(format)))
    }

    pub fn into_error(self) -> RekallError {
        let kind = match self.kind {
            ExportFailureKind::Render => "Render",
            ExportFailureKind::Storage => "Storage",
        };
        RekallError::internal(InternalFailure::Export, format!("{kind} error. {}", self.message))
    }

    fn with(kind: ExportFailureKind, message: String) -> Self {
        Self { kind, message: capped(message) }
    }
}

impl fmt::Display for ExportFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} failure: {}", self.kind, self.message)
    }
}

fn capped(message: String) -> String {
    if jstr::len(&message) <= FAILURE_MESSAGE_MAX_CHARACTERS {
        return message;
    }
    format!("{}…", jstr::prefix(&message, FAILURE_MESSAGE_MAX_CHARACTERS - 1))
}

#[cfg(test)]
#[path = "../../../tests/unit/export/rule/export_failure_tests.rs"]
mod tests;
