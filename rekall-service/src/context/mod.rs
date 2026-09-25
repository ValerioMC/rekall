//! What Rekall does at read time: resolve anchors, walk the associations in one read-only
//! transaction into a materialised tree of `ContextRecord`s, and render that tree as the markdown
//! a session reads. The renderer is also what the console measures a task's context with, so the
//! size shown is the length of what a session receives.

mod record;
mod renderer;
mod service;
mod size;

pub use record::{ContextCommitView, ContextRecord, DocumentView};
pub use renderer::{ContextRenderer, MAX_DIFF_CHARACTERS, MAX_DOCUMENT_CHARACTERS, REFERENCE_LINE_MAX};
pub use service::{ContextService, ENTITY_NAMES};
pub use size::{ContextSize, ContextSizePart, ContextSizeService, CHARACTERS_PER_TOKEN};
