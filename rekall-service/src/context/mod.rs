//! What Rekall does at read time: resolve anchors, walk the associations in one read-only
//! transaction into a materialised tree of `ContextRecord`s, and render that tree as the markdown
//! a session reads. The renderer is also what the console measures a task's context with, so the
//! size shown is the length of what a session receives.

mod context_commit_view;
mod context_record;
mod context_renderer;
mod context_service;
mod context_size;
mod context_size_part;
mod context_size_service;
mod document_view;

pub use context_commit_view::ContextCommitView;
pub use context_record::ContextRecord;
pub use context_renderer::{ContextRenderer, MAX_DIFF_CHARACTERS, MAX_DOCUMENT_CHARACTERS, REFERENCE_LINE_MAX};
pub use context_service::{ContextService, ENTITY_NAMES};
pub use context_size::ContextSize;
pub use context_size_part::ContextSizePart;
pub use context_size_service::{estimate_tokens, ContextSizeService, CHARACTERS_PER_TOKEN};
pub use document_view::DocumentView;
