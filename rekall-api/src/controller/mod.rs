//! One router per Java controller, under the same paths.

mod api_router;
mod catalog_controller;
mod commit_reference_controller;
mod company_controller;
mod context_size_controller;
mod directory_listing_controller;
mod document_controller;
mod export_controller;
mod responses;
mod search_controller;
mod step_event_controller;
mod tag_controller;
mod task_revision_controller;
mod task_step_controller;
mod time_entry_controller;
mod wrapup_controller;

pub use api_router::{router, routes};
pub use responses::{created, no_content};
