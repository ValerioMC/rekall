//! Time tracking: one session of work on a task at a time per task, corrected by hand within
//! rules that keep it sane.

mod service;
mod view;

pub use service::time_entry_service::{TimeEntryService, IDLE_AFTER};
pub use view::time_entry_view::TimeEntryView;
