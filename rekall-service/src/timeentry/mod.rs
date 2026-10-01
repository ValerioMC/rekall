//! Time tracking: one session of work on a task at a time per task, corrected by hand within
//! rules that keep it sane.

mod time_entry_service;
mod time_entry_view;

pub use time_entry_service::{TimeEntryService, IDLE_AFTER};
pub use time_entry_view::TimeEntryView;
