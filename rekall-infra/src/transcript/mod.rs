//! Claude Code's session transcripts, read for the tokens each session spent on Rekall's tasks
//! and steps. `SessionLedger` decides what each model call is charged to, `TranscriptFolder`
//! finds the sessions of a project's folder, and `TranscriptScanner` reads the ones that grew.

mod session_files;
mod session_ledger;
mod transcript_entry;
mod transcript_folder;
mod transcript_scanner;
mod work_terms;

pub use session_files::SessionFiles;
pub use session_ledger::SessionLedger;
pub(crate) use transcript_folder::{flatten, folder_name};
pub use transcript_folder::TranscriptFolder;
pub use transcript_scanner::TranscriptScanner;
