//! Earlier versions of a task's wrapup and description, so replacing or deleting one is never the
//! end of it. Whatever is about to overwrite the text calls `keep` with the text about to go:
//!
//! - Blank text, or text identical to the newest revision, is not kept.
//! - A hand edit in the console autosaves as it is typed, so a hand edit over hand-written text
//!   keeps one revision per ten-minute window: the version from before the editing started. A
//!   hand edit over a session's text is always kept, since that text was nobody's draft.
//! - A session's write, a deletion and a restore are discrete events and are always kept.
//! - Only the newest thirty revisions of each text are kept per task.

mod restored_revision;
mod revision_trigger;
mod task_revision_service;
mod task_revision_view;

pub use restored_revision::RestoredRevision;
pub use revision_trigger::RevisionTrigger;
pub use task_revision_service::{KEPT_PER_KIND, hand_edit_window, TaskRevisionService};
pub use task_revision_view::TaskRevisionView;
