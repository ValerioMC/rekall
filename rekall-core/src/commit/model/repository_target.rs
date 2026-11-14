use std::path::{Path, PathBuf};

use rekall_kernel::{jstr, Id};
use rekall_store::{project, project_component, task};

/// A repository a task can change: the project's own folder (no component) or the folder of one
/// of its components. A component folder inside the project's folder is the same repository, so
/// its working tree is never read twice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryTarget {
    pub component_id: Option<Id>,
    pub name: String,
    pub folder: Option<PathBuf>,
    pub reads_own_working_tree: bool,
}

impl RepositoryTarget {
    /// The project's own repository first, then one target per component, in the console's order.
    pub fn all_of(task: &task::Model, project: &project::Model, components: &[project_component::Model]) -> Vec<Self> {
        let project_folder = project.repo_folder.as_deref().filter(|folder| !jstr::is_blank(folder)).map(|folder| PathBuf::from(jstr::strip(folder)));
        let mut targets = vec![Self::primary(task, project, project_folder.as_deref())];
        targets.extend(components.iter().map(|component| Self::component(component, project_folder.as_deref())));
        targets
    }

    fn primary(task: &task::Model, project: &project::Model, project_folder: Option<&Path>) -> Self {
        let name = project_folder.and_then(Path::file_name).map_or_else(|| project.title.clone(), |name| name.to_string_lossy().into_owned());
        let folder = task.work_folder(project).map(|folder| PathBuf::from(jstr::strip(folder)));
        Self { component_id: None, name, folder, reads_own_working_tree: true }
    }

    fn component(component: &project_component::Model, project_folder: Option<&Path>) -> Self {
        let folder = PathBuf::from(jstr::strip(&component.folder));
        let inside_project_folder = project_folder.is_some_and(|project_folder| folder.starts_with(project_folder));
        Self { component_id: Some(component.id), name: component.name.clone(), folder: Some(folder), reads_own_working_tree: !inside_project_folder }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/commit/model/repository_target_tests.rs"]
mod tests;
