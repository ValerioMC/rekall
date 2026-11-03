use rekall_kernel::{Id, Instant};
use rekall_store::ProjectStatus;

use super::*;

fn project(label: &str, repo_folder: Option<&str>) -> project::Model {
    let now = Instant::now();
    project::Model {
        id: Id::random(),
        label: label.into(),
        title: label.into(),
        status: ProjectStatus::Active,
        icon: String::new(),
        description: None,
        blueprint_markdown: None,
        repo_folder: repo_folder.map(String::from),
        auto_commit: false,
        company_id: Id::random(),
        created_at: now,
        updated_at: now,
    }
}

fn label_holding(projects: &[project::Model], folder: &str) -> Option<String> {
    project_holding(projects, &[], folder).map(|project| project.label.clone())
}

#[test]
fn a_folder_inside_a_projects_folder_belongs_to_that_project() {
    let projects = [project("vega", Some("/work/vega")), project("orion", Some("/work/orion"))];

    assert_eq!(label_holding(&projects, "/work/vega/rekall-core/src"), Some("vega".into()));
    assert_eq!(label_holding(&projects, "/work/orion"), Some("orion".into()));
}

#[test]
fn the_innermost_project_folder_wins() {
    let projects = [project("mono", Some("/work/mono")), project("mono-ui", Some("/work/mono/ui"))];

    assert_eq!(label_holding(&projects, "/work/mono/ui/src"), Some("mono-ui".into()));
    assert_eq!(label_holding(&projects, "/work/mono/api"), Some("mono".into()));
}

#[test]
fn a_folder_that_only_shares_a_prefix_is_not_inside() {
    let projects = [project("vega", Some("/work/vega"))];

    assert_eq!(label_holding(&projects, "/work/vega-old/src"), None);
}

#[test]
fn a_project_without_a_folder_holds_nothing() {
    let projects = [project("vega", None), project("blank", Some("  "))];

    assert_eq!(label_holding(&projects, "/work/vega"), None);
}

#[test]
fn a_folder_inside_a_tasks_worktree_belongs_to_the_tasks_project() {
    let projects = [project("vega", Some("/work/vega"))];
    let mut task = task::Model::new("report".into(), "Report".into(), projects[0].id, Instant::now());
    task.worktree_folder = Some("/data/worktrees/vega/report".into());

    let holder = project_holding(&projects, &[task], "/data/worktrees/vega/report/rekall-core");

    assert_eq!(holder.map(|project| project.label.as_str()), Some("vega"));
}
