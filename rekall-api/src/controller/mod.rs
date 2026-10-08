//! One router per resource.

mod api_router;
mod commit_reference_controller;
mod company_controller;
mod company_logo_controller;
mod context_size_controller;
mod diagram_controller;
mod directory_listing_controller;
mod document_controller;
mod project_component_controller;
mod project_controller;
mod project_icon_controller;
mod project_section_controller;
mod responses;
mod search_controller;
mod step_event_controller;
mod tag_controller;
mod task_controller;
mod task_revision_controller;
mod task_step_controller;
mod time_entry_controller;
mod wrapup_controller;

pub use api_router::{router, routes};
pub use responses::{created, no_content, png};
