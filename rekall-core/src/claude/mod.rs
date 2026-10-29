//! What the terminals and the run queue in `rekall-infra` need from the entities, read here so
//! that crate never touches a repository; the models and efforts a session may start with;
//! `ResourcePath`, the only way to name a file of Claude Code's home folder, and what the console
//! is shown of that folder and of the account's usage; and the ports the machine is reached
//! through: `TerminalHost` and `TerminalPipe` for the terminals, `UsageReader` for the usage
//! meter, `ClaudeResources` for the home folder and `ClaudeCodeSetup` for the install.

mod model;
mod port;
mod service;
mod view;

pub use port::claude_code_setup::ClaudeCodeSetup;
pub use port::claude_resources::ClaudeResources;
pub use view::claude_usage_view::ClaudeUsageView;
pub use model::covering_document::{CoveringDocument, CoveringKind};
pub use model::installation::Installation;
pub use model::launch_choices::{EFFORT_LEVELS, MODEL_ALIASES};
pub use model::limit::Limit;
pub use model::memory_analysis::MemoryAnalysis;
pub use model::memory_group::MemoryGroup;
pub use model::memory_ref::MemoryRef;
pub use model::obsolete_memory::ObsoleteMemory;
pub use model::promotion_candidate::PromotionCandidate;
pub use model::promotion_target::PromotionTarget;
pub use model::remaining::Remaining;
pub use model::resource_catalog::ResourceCatalog;
pub use model::resource_file::ResourceFile;
pub use model::resource_kind::ResourceKind;
pub use model::resource_path::ResourcePath;
pub use model::resource_save::ResourceSave;
pub use model::resource_summary::ResourceSummary;
pub use model::severity::Severity;
pub use model::skill_source::{SkillSource, SkillSourceKind};
pub use model::task_work::TaskWork;
pub use service::task_work_service::TaskWorkService;
pub use model::terminal_connection::TerminalConnection;
pub use view::terminal_connection_view::TerminalConnectionView;
pub use port::terminal_host::TerminalHost;
pub use model::terminal_launch::TerminalLaunch;
pub use service::terminal_launch_service::TerminalLaunchService;
pub use port::terminal_listener::TerminalListener;
pub use model::terminal_mode::{TerminalMode, REQUEST_MAX_CHARACTERS};
pub use model::terminal_opening::TerminalOpening;
pub use port::terminal_pipe::TerminalPipe;
pub use model::terminal_target::TerminalTarget;
pub use view::terminal_view::TerminalView;
pub use port::usage_reader::UsageReader;
pub use model::usage_status::UsageStatus;
