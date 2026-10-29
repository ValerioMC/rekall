//! What the terminal and the run queue in `rekall-infra` need from the entities, read here so
//! that crate never touches a repository; the models and efforts a session may start with; and
//! `ResourcePath`, the only way to name a file of Claude Code's home folder.

mod launch_choices;
mod remaining;
mod resource_kind;
mod resource_path;
mod task_work;
mod task_work_service;
mod terminal_connection;
mod terminal_launch;
mod terminal_launch_service;
mod terminal_target;

pub use launch_choices::{EFFORT_LEVELS, MODEL_ALIASES};
pub use remaining::Remaining;
pub use resource_kind::ResourceKind;
pub use resource_path::ResourcePath;
pub use task_work::TaskWork;
pub use task_work_service::TaskWorkService;
pub use terminal_connection::TerminalConnection;
pub use terminal_launch::TerminalLaunch;
pub use terminal_launch_service::TerminalLaunchService;
pub use terminal_target::TerminalTarget;
