//! The routes of the Claude screen and the meter: the account's usage, the files of Claude Code's
//! home folder, how the person groups their skills, and whether Claude Code knows Rekall. Served by
//! `rekall_console::claude` and, for skill groups, `rekall_core::skill`.

mod installer_controller;
mod resource_controller;
mod resource_query;
mod routes;
mod save_request;
mod skill_group_controller;
mod skill_group_requests;
mod usage_controller;
mod usage_query;

pub use save_request::SaveRequest;
pub use skill_group_requests::{PlaceSkillRequest, RenameGroupRequest};
pub(crate) use routes::routes;
