//! The one way a session writes a note on one task. It adds a new one, or, asked to replace, rewrites
//! the body of the note that task carries under the same title. It never moves, detaches or deletes
//! one; the console's document service in rekall-console stays the only path for those.

mod event;
mod model;
mod service;

pub use service::note_service::{KIND, TITLE_MAX_CHARACTERS, BODY_MAX_CHARACTERS, NoteService};
pub use event::note_stream_event::NoteStreamEvent;
pub use model::on_title_clash::OnTitleClash;
pub use model::written::Written;
