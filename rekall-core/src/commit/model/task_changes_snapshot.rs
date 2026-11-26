use std::sync::Arc;

use rekall_kernel::{Id, RekallError, Result};

use super::change_file_key::ChangeFileKey;
use super::indexed_patch::IndexedPatch;
use super::task_changes_scope::TaskChangesScope;
use crate::commit::view::{ChangeFileHunksView, ChangeSourceId, RepositoryChangesView, TaskChangesIndexView};

pub(in crate::commit) struct TaskChangesSnapshot {
    pub id: Id,
    pub task_id: Id,
    repositories: Vec<RepositoryChangesView<Arc<IndexedPatch>>>,
}

impl TaskChangesSnapshot {
    pub fn new(task_id: Id, repositories: Vec<RepositoryChangesView<Arc<IndexedPatch>>>) -> Self {
        Self {
            id: Id::random(),
            task_id,
            repositories,
        }
    }

    pub fn index(&self) -> TaskChangesIndexView {
        let repositories = self
            .repositories
            .iter()
            .cloned()
            .map(|repo| repo.map_files(|patch| Some(patch.metadata())))
            .collect();
        TaskChangesIndexView {
            snapshot_id: self.id,
            repositories,
        }
    }

    pub fn hunks(&self, scope: TaskChangesScope, files: &[ChangeFileKey]) -> Result<Vec<ChangeFileHunksView>> {
        let found: Vec<_> = self
            .repositories
            .iter()
            .flat_map(|repo| repository_hunks(repo, scope, files))
            .collect();
        if let Some(file) = files.iter().find(|file| !found.iter().any(|entry| &entry.file == *file)) {
            return Err(RekallError::illegal(format!(
                "File {:?} is not indexed in the selected Changes scope",
                file.path
            )));
        }
        Ok(found)
    }
}

fn repository_hunks(
    repo: &RepositoryChangesView<Arc<IndexedPatch>>,
    scope: TaskChangesScope,
    files: &[ChangeFileKey],
) -> Vec<ChangeFileHunksView> {
    let keys: Vec<_> = files.iter().filter(|file| file.component_id == repo.component_id).collect();
    if keys.is_empty() {
        return Vec::new();
    }
    let mut found = Vec::new();
    for commit in &repo.commits {
        if scope.includes_commit(repo.component_id, commit.reference_id, commit.step_id) {
            found.extend(source_hunks(
                &commit.files,
                ChangeSourceId::Commit {
                    reference_id: commit.reference_id,
                },
                &keys,
            ));
        }
    }
    if let Some(tree) = repo
        .working_tree
        .as_ref()
        .filter(|_| scope.includes_working_tree(repo.component_id))
    {
        found.extend(source_hunks(&tree.files, ChangeSourceId::WorkingTree, &keys));
    }
    found
}

fn source_hunks(patch: &IndexedPatch, source: ChangeSourceId, files: &[&ChangeFileKey]) -> Vec<ChangeFileHunksView> {
    files
        .iter()
        .filter_map(|file| {
            patch.hunks(&file.path).map(|hunks| ChangeFileHunksView {
                file: (*file).clone(),
                source,
                hunks,
            })
        })
        .collect()
}
