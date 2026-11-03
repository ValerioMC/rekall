//! The one write path for a task's wrapup. A write replaces the text outright and keeps the text it
//! wrote as a version, in the same transaction; a delete removes the wrapup and leaves its versions.

mod event;
mod model;
mod service;
mod view;

pub use service::wrapup_freshness_service::WrapupFreshnessService;
pub use service::wrapup_service::WrapupService;
pub use model::freshness::Freshness;
pub use model::named_paths::{NamedPaths, MAX_NAMED_PATHS};
pub use event::wrapup_stream_event::WrapupStreamEvent;
pub use view::wrapup_version_view::{WrapupVersionDetail, WrapupVersionView};
pub use view::wrapup_view::WrapupView;
pub use model::written::Written;
pub use rekall_store::WrapupAuthor;
