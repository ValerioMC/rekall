//! What Rekall does at read time: resolve anchors, walk the associations in one read-only
//! transaction into a materialised tree of `ContextRecord`s, and render that tree as the markdown
//! a session reads. The renderer is also what the console measures a task's context with, so the
//! size shown is the length of what a session receives.

mod model;
mod service;
mod view;

pub use view::context_commit_view::ContextCommitView;
pub use model::context_record::ContextRecord;
pub use model::context_renderer::{ContextRenderer, MAX_DIFF_CHARACTERS, MAX_DOCUMENT_CHARACTERS, REFERENCE_LINE_MAX};
pub use service::context_service::{ContextService, ENTITY_NAMES};
pub use model::context_size::ContextSize;
pub use model::context_size_part::ContextSizePart;
pub use view::context_skill_view::{ContextSkillScope, ContextSkillView};
pub use service::context_size_service::{estimate_tokens, ContextSizeService, CHARACTERS_PER_TOKEN};
pub use view::document_view::DocumentView;
pub use view::step_connection_view::StepConnectionView;
