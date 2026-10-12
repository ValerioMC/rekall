use std::path::PathBuf;

use rekall_common::{Instant, RekallError, Result};

use super::{memory_index, ClaudeHome, Frontmatter, MemoryAnalysis, MemoryAnalyzer, ResourceCatalog, ResourceFile, ResourcePath, ResourceScanner, ResourceSummary, SaveRequest};

/// The most a file written from the console may hold.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Reads and edits Claude Code's own files: instructions, skills, commands, agents and each
/// project's memories. The disk is the record and a session may rewrite a file at any moment, so
/// every write names the version it was made from.
#[derive(Clone, Debug)]
pub struct ClaudeResourceService {
    home: ClaudeHome,
}

impl ClaudeResourceService {
    pub fn new(root: PathBuf) -> Self {
        Self { home: ClaudeHome::new(root) }
    }

    pub fn catalog(&self) -> ResourceCatalog {
        ResourceScanner::new(&self.home).catalog()
    }

    /// Which memories keep coming back across projects and which a `CLAUDE.md` or skill repeats.
    pub fn analyze(&self) -> MemoryAnalysis {
        MemoryAnalyzer::new(&self.home).analyze(&self.catalog())
    }

    pub fn read(&self, raw_path: &str) -> Result<ResourceFile> {
        let path = ResourcePath::parse(raw_path)?;
        self.open(&path)
    }

    pub fn save(&self, request: SaveRequest) -> Result<ResourceFile> {
        let path = ResourcePath::parse(request.path.as_deref().unwrap_or_default())?;
        let body = request.body.unwrap_or_default();
        ensure_editable(&path)?;
        ensure_fits(&body)?;
        let current = self.home.modified_at(&path)?;
        ensure_same_version(&path, current, request.expected_modified_at)?;
        self.home.write(&path, &body)?;
        if current.is_none() {
            self.index_new_memory(&path, &body)?;
        }
        self.open(&path)
    }

    pub fn delete(&self, raw_path: &str) -> Result<()> {
        let path = ResourcePath::parse(raw_path)?;
        ensure_editable(&path)?;
        if !path.kind().is_deletable() {
            return Err(RekallError::conflict(format!("{} can be edited but not deleted.", path.file_name())));
        }
        self.home.remove(&path)?;
        self.unindex_memory(&path)
    }

    fn open(&self, path: &ResourcePath) -> Result<ResourceFile> {
        let body = self.home.read(path)?;
        Ok(ResourceFile { summary: ResourceSummary::read(&self.home, path)?, body })
    }

    /// A new memory gets a line in the project's `MEMORY.md`, the file Claude loads to find it.
    fn index_new_memory(&self, path: &ResourcePath, body: &str) -> Result<()> {
        let Some(index) = path.memory_index() else { return Ok(()) };
        let front = Frontmatter::parse(body);
        let stem = path.file_name().trim_end_matches(".md");
        let title = front.name.as_deref().unwrap_or(stem);
        let current = self.index_body(&index)?;
        let updated = memory_index::with_entry(&current, path.file_name(), title, front.description.as_deref().unwrap_or_default());
        self.write_index(&index, &current, updated)
    }

    fn unindex_memory(&self, path: &ResourcePath) -> Result<()> {
        let Some(index) = path.memory_index() else { return Ok(()) };
        let current = self.index_body(&index)?;
        let updated = memory_index::without_entry(&current, path.file_name());
        self.write_index(&index, &current, updated)
    }

    fn index_body(&self, index: &ResourcePath) -> Result<String> {
        match self.home.read(index) {
            Err(RekallError::NotFound(_)) => Ok(String::new()),
            other => other,
        }
    }

    fn write_index(&self, index: &ResourcePath, current: &str, updated: String) -> Result<()> {
        if updated == current {
            return Ok(());
        }
        self.home.write(index, &updated)
    }
}

fn ensure_editable(path: &ResourcePath) -> Result<()> {
    if path.is_editable() {
        return Ok(());
    }
    Err(RekallError::conflict(format!("{} is provided by claude.ai or a plugin and is overwritten when it updates.", path.as_str())))
}

fn ensure_fits(body: &str) -> Result<()> {
    if body.len() <= MAX_BODY_BYTES {
        return Ok(());
    }
    Err(RekallError::illegal(format!("A file is capped at {MAX_BODY_BYTES} bytes; this one is {}.", body.len())))
}

/// An existing file is rewritten only from the version the editor opened; a new one only where
/// nothing is. Versions compare to the millisecond, the precision the wire carries.
fn ensure_same_version(path: &ResourcePath, current: Option<Instant>, expected: Option<Instant>) -> Result<()> {
    match (current, expected) {
        (None, None) => Ok(()),
        (Some(found), Some(opened)) if found.epoch_millis() == opened.epoch_millis() => Ok(()),
        (Some(_), Some(_)) => Err(RekallError::conflict(format!("{} changed on disk since it was opened. Reload it before saving.", path.as_str()))),
        (Some(_), None) => Err(RekallError::conflict(format!("{} already exists.", path.as_str()))),
        (None, Some(_)) => Err(RekallError::conflict(format!("{} was removed since it was opened.", path.as_str()))),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/resources/claude_resource_service_tests.rs"]
mod tests;

