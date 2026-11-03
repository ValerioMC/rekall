use std::path::{Path, PathBuf};

use rekall_kernel::{RekallError, Result};

pub const BRANCH_PREFIX: &str = "rekall/";

/// Where a task's worktree lives and what its branch is called: under `root`, outside every
/// repository, one folder per project and task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeLayout {
    pub root: PathBuf,
}

impl WorktreeLayout {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn branch_for(&self, task_label: &str) -> Result<String> {
        require_safe_name("task label", task_label)?;
        Ok(format!("{BRANCH_PREFIX}{task_label}"))
    }

    pub fn folder_for(&self, project_label: &str, task_label: &str) -> Result<PathBuf> {
        require_safe_name("project label", project_label)?;
        require_safe_name("task label", task_label)?;
        Ok(self.root.join(project_label).join(task_label))
    }

    /// A branch this layout names: `rekall/` and a safe task label.
    pub fn require_branch(&self, branch_name: &str) -> Result<()> {
        let Some(task_label) = branch_name.strip_prefix(BRANCH_PREFIX) else {
            return Err(RekallError::illegal(format!("'{branch_name}' is not a branch Rekall names: it starts with '{BRANCH_PREFIX}'.")));
        };
        require_safe_name("branch name", task_label)
    }

    /// A folder under the root with no `..` in it. Symbolic links are the host's to resolve.
    pub fn require_folder(&self, folder: &Path) -> Result<()> {
        let relative = folder
            .strip_prefix(&self.root)
            .map_err(|_| RekallError::illegal(format!("'{}' is outside the worktree folder.", folder.display())))?;
        let escapes = relative.components().any(|part| !matches!(part, std::path::Component::Normal(_)));
        if relative.as_os_str().is_empty() || escapes {
            return Err(RekallError::illegal(format!("'{}' is not a folder inside the worktree folder.", folder.display())));
        }
        Ok(())
    }
}

fn require_safe_name(what: &str, name: &str) -> Result<()> {
    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.');
    if name.is_empty() || name.starts_with(['.', '-']) || name.contains("..") || !name.chars().all(allowed) {
        return Err(RekallError::illegal(format!(
            "'{name}' is not a usable {what}: lowercase letters, digits, '-', '_' and '.' only, and it cannot start with '.' or '-'."
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/rule/worktree_layout_tests.rs"]
mod tests;
