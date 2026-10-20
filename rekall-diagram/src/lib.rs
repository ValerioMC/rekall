//! A diagram as a session writes it: one or more Mermaid sheets, each tying the elements it
//! draws to the code that implements them, and optionally a walkthrough to play a sequence by.
//!
//! It knows nothing of the database or the console. A session sends it as JSON (`format`),
//! the application checks it (`validation`) and stores it as text, and the console draws it.
//! `mermaid` reads just enough of a sheet's text to hold its links to what it declares.

pub mod document;
pub mod format;
pub mod mermaid;
pub mod validation;

pub use document::{DiagramDocument, DiagramKind, DiagramSheet, SourceLocation, Walkthrough, WalkthroughMessage};
pub use format::{DocumentCodec, DocumentFormatError};
pub use mermaid::{ElementKey, MermaidText};
pub use validation::{DocumentValidator, DocumentViolation, DocumentViolations};
