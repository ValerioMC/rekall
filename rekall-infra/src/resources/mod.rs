//! Claude Code's home folder as the console edits it: `CLAUDE.md`, skills, commands, agents and
//! each project's memories. No table: the disk is the record, `ResourcePath` is the only way to
//! name a file, and every write names the version it was made from.

mod claude_home;
mod claude_resource_service;
mod frontmatter;
mod memory_analyzer;
mod memory_index;
mod reference_library;
mod resource_scanner;
mod resource_summary_reader;
mod plugin_skills;
mod synced_sources;
mod text_fingerprint;
mod working_directory;

pub use claude_home::ClaudeHome;
pub use claude_resource_service::ClaudeResourceService;
pub use frontmatter::Frontmatter;

use memory_analyzer::MemoryAnalyzer;
use rekall_core::claude::{
    CoveringDocument, CoveringKind, MemoryAnalysis, MemoryGroup, MemoryRef, ObsoleteMemory, PromotionCandidate, PromotionTarget, ResourceCatalog,
    ResourceFile, ResourceKind, ResourcePath, ResourceSave, ResourceSummary, SkillSource,
};
use resource_summary_reader::read_summary;
use reference_library::ReferenceLibrary;
use resource_scanner::ResourceScanner;
use synced_sources::SyncedSources;
use text_fingerprint::TextFingerprint;
use working_directory::WorkingDirectory;
