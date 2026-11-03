use std::path::PathBuf;

/// A working folder of `project_folder`'s repository, on `branch`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worktree {
    pub project_folder: PathBuf,
    pub folder: PathBuf,
    pub branch: String,
}
