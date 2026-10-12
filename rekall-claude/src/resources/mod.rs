//! Claude Code's home folder as the console edits it: `CLAUDE.md`, skills, commands, agents and
//! each project's memories. No table: the disk is the record, `ResourcePath` is the only way to
//! name a file, and every write names the version it was made from.

mod claude_home;
mod claude_resource_service;
mod frontmatter;
mod covering_document;
mod memory_analysis;
mod memory_analyzer;
mod memory_group;
mod memory_index;
mod memory_ref;
mod obsolete_memory;
mod promotion_candidate;
mod promotion_target;
mod reference_library;
mod resource_catalog;
mod resource_controller;
mod resource_file;
mod resource_kind;
mod resource_path;
mod resource_query;
mod resource_scanner;
mod resource_summary;
mod plugin_skills;
mod save_request;
mod skill_group_controller;
mod skill_group_requests;
mod skill_group_service;
mod skill_placement_view;
mod skill_source;
mod synced_sources;
mod text_fingerprint;
mod working_directory;

pub use claude_home::ClaudeHome;
pub use covering_document::{CoveringDocument, CoveringKind};
pub use memory_analysis::MemoryAnalysis;
pub use memory_ref::MemoryRef;
pub use obsolete_memory::ObsoleteMemory;
pub use promotion_candidate::PromotionCandidate;
pub use promotion_target::PromotionTarget;
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
pub use skill_group_service::SkillGroupService;
pub use skill_placement_view::SkillPlacementView;
pub use skill_source::{SkillSource, SkillSourceKind};

use memory_analyzer::MemoryAnalyzer;
use reference_library::ReferenceLibrary;
use resource_scanner::ResourceScanner;
use synced_sources::SyncedSources;
use text_fingerprint::TextFingerprint;
use working_directory::WorkingDirectory;
