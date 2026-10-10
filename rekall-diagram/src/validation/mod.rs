//! The rules a document has to satisfy before it is stored. Every broken rule is reported, each
//! with the path to what broke it, so a session can fix them all in one pass.

mod document_validator;
mod document_violation;
mod document_violations;

pub use document_validator::{DocumentValidator, MAX_SHEETS};
pub use document_violation::DocumentViolation;
pub use document_violations::DocumentViolations;
