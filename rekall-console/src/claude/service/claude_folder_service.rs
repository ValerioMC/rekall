use std::sync::Arc;

use rekall_core::claude::{ClaudeResources, MemoryAnalysis, ResourceCatalog, ResourceFile, ResourceSave};
use rekall_kernel::{InternalFailure, RekallError, Result};

use crate::claude::SaveResourceCommand;

/// The files of Claude Code's home folder. The disk is read and written with blocking calls, so
/// each runs off the async workers.
#[derive(Clone)]
pub struct ClaudeFolderService {
    resources: Arc<dyn ClaudeResources>,
}

impl ClaudeFolderService {
    pub fn new(resources: Arc<dyn ClaudeResources>) -> Self {
        Self { resources }
    }

    pub async fn catalog(&self) -> Result<ResourceCatalog> {
        let resources = self.resources.clone();
        blocking(move || Ok(resources.catalog())).await
    }

    pub async fn analyze(&self) -> Result<MemoryAnalysis> {
        let resources = self.resources.clone();
        blocking(move || Ok(resources.analyze())).await
    }

    pub async fn read(&self, path: String) -> Result<ResourceFile> {
        let resources = self.resources.clone();
        blocking(move || resources.read(&path)).await
    }

    pub async fn save(&self, command: SaveResourceCommand) -> Result<ResourceFile> {
        let resources = self.resources.clone();
        let save = ResourceSave { path: command.path, body: command.body, expected_modified_at: command.expected_modified_at };
        blocking(move || resources.save(save)).await
    }

    pub async fn delete(&self, path: String) -> Result<()> {
        let resources = self.resources.clone();
        blocking(move || resources.delete(&path)).await
    }
}

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(work).await.map_err(|failed| RekallError::internal(InternalFailure::Background, failed))?
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/service/claude_folder_service_tests.rs"]
mod tests;
