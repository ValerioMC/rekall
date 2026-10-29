//! The routes of the Claude screen and the meter: the account's usage, the files of Claude Code's
//! home folder, how the person groups their skills, and whether Claude Code knows Rekall. Served by
//! `rekall_console::claude` and, for skill groups, `rekall_core::skill`.

mod controller;
mod request;
mod routes;

pub use request::save_request::SaveRequest;
pub use request::skill_group_requests::{PlaceSkillRequest, RenameGroupRequest};
pub(crate) use routes::routes;
