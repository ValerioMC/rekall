//! Claude Code's home folder as the console edits it: `CLAUDE.md`, skills, commands, agents and
//! each project's memories. No table: the disk is the record, `ResourcePath` is the only way to
//! name a file, and every write names the version it was made from.

mod claude_home;
mod claude_resource_service;
mod frontmatter;
mod memory_group;
mod memory_index;
mod resource_catalog;
mod resource_controller;
mod resource_file;
mod resource_kind;
mod resource_path;
mod resource_query;
mod resource_scanner;
mod resource_summary;
mod save_request;
mod working_directory;

pub use claude_home::ClaudeHome;
pub use claude_resource_service::ClaudeResourceService;
pub use frontmatter::Frontmatter;
pub use memory_group::MemoryGroup;
pub use resource_catalog::ResourceCatalog;
pub use resource_controller::routes;
pub use resource_file::ResourceFile;
pub use resource_kind::ResourceKind;
pub use resource_path::ResourcePath;
pub use resource_summary::ResourceSummary;
pub use save_request::SaveRequest;

use resource_scanner::ResourceScanner;
use working_directory::WorkingDirectory;
