use std::collections::HashMap;
use std::sync::Mutex;

use rekall_core::claude::{ResourceKind, ResourceSummary};
use rekall_kernel::Instant;

use super::*;

/// A folder held in memory: a file exists once saved, and a save that names a version only lands
/// on a file that exists.
#[derive(Default)]
struct FakeFolder {
    files: Mutex<HashMap<String, String>>,
}

fn file(path: &str, body: &str) -> ResourceFile {
    let summary = ResourceSummary {
        path: path.into(),
        kind: ResourceKind::Memory,
        name: path.into(),
        description: None,
        memory_type: None,
        bytes: body.len() as u64,
        modified_at: Instant::now(),
        editable: true,
        source: None,
    };
    ResourceFile { summary, body: body.into() }
}

impl ClaudeResources for FakeFolder {
    fn catalog(&self) -> ResourceCatalog {
        ResourceCatalog { root: "/home/.claude".into(), instructions: None, skills: Vec::new(), commands: Vec::new(), agents: Vec::new(), memory_groups: Vec::new() }
    }

    fn analyze(&self) -> MemoryAnalysis {
        MemoryAnalysis { memories_read: self.files.lock().unwrap().len(), documents_read: 0, promotions: Vec::new(), obsolete: Vec::new() }
    }

    fn read(&self, path: &str) -> Result<ResourceFile> {
        let files = self.files.lock().unwrap();
        files.get(path).map(|body| file(path, body)).ok_or_else(|| RekallError::not_found_msg(format!("No file {path}")))
    }

    fn save(&self, save: ResourceSave) -> Result<ResourceFile> {
        let path = save.path.unwrap_or_default();
        let mut files = self.files.lock().unwrap();
        if save.expected_modified_at.is_some() != files.contains_key(&path) {
            return Err(RekallError::conflict("The file changed since it was opened."));
        }
        let body = save.body.unwrap_or_default();
        files.insert(path.clone(), body.clone());
        Ok(file(&path, &body))
    }

    fn delete(&self, path: &str) -> Result<()> {
        self.files.lock().unwrap().remove(path).map(|_| ()).ok_or_else(|| RekallError::not_found_msg(format!("No file {path}")))
    }
}

fn service() -> ClaudeFolderService {
    ClaudeFolderService::new(Arc::new(FakeFolder::default()))
}

fn command(path: &str, body: &str, expected: Option<Instant>) -> SaveResourceCommand {
    SaveResourceCommand { path: Some(path.into()), body: Some(body.into()), expected_modified_at: expected }
}

#[tokio::test]
async fn a_saved_file_reads_back_counts_in_the_analysis_and_goes_once_deleted() {
    let service = service();

    service.save(command("MEMORY.md", "- one", None)).await.unwrap();

    assert_eq!(service.read("MEMORY.md".into()).await.unwrap().body, "- one");
    assert_eq!(service.analyze().await.unwrap().memories_read, 1);
    service.delete("MEMORY.md".into()).await.unwrap();
    assert!(matches!(service.read("MEMORY.md".into()).await, Err(RekallError::NotFound(_))));
}

#[tokio::test]
async fn the_version_the_editor_opened_travels_with_the_save() {
    let service = service();

    let refused = service.save(command("CLAUDE.md", "rules", Some(Instant::now()))).await;

    assert!(matches!(refused, Err(RekallError::Conflict(_))));
    assert_eq!(service.catalog().await.unwrap().root, "/home/.claude");
}
