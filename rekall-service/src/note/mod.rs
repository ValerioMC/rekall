//! The one way a session writes a note on one task. It adds a new one, or, asked to replace, rewrites
//! the body of the note that task carries under the same title. It never moves, detaches or deletes
//! one; the console's document service in rekall-api stays the only path for those.

mod note_service;
mod note_stream_event;
mod on_title_clash;
mod written;

pub use note_service::{KIND, TITLE_MAX_CHARACTERS, BODY_MAX_CHARACTERS, NoteService};
pub use note_stream_event::NoteStreamEvent;
pub use on_title_clash::OnTitleClash;
pub use written::Written;
