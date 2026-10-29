//! Whether a newer release than this build exists, read through the `ReleaseFeed` port of
//! `rekall_core::version`.

mod update_check_service;

pub use update_check_service::UpdateCheckService;
