use rekall_kernel::Result;

use super::{MemoryAnalysis, ResourceCatalog, ResourceFile, ResourceSave};

/// Claude Code's home folder: its files listed, analysed, read, written and deleted. Every call
/// reads or writes the disk and blocks.
pub trait ClaudeResources: Send + Sync {
    fn catalog(&self) -> ResourceCatalog;

    fn analyze(&self) -> MemoryAnalysis;

    fn read(&self, path: &str) -> Result<ResourceFile>;

    fn save(&self, save: ResourceSave) -> Result<ResourceFile>;

    fn delete(&self, path: &str) -> Result<()>;
}
