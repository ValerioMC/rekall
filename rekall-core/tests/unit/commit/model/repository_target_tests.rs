use super::*;
use rekall_kernel::Instant;
use rekall_store::ProjectStatus;

fn project_in(folder: Option<&str>) -> project::Model {
    let now = Instant::now();
    project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(), description: None,
        blueprint_markdown: None, repo_folder: folder.map(str::to_string), auto_commit: false, main_branch: None, pull_request_flow: false,
        task_key: String::new(), company_id: Id::random(), created_at: now, updated_at: now,
    }
}

fn component(name: &str, folder: &str, project: &project::Model) -> project_component::Model {
    let now = Instant::now();
    project_component::Model { id: Id::random(), project_id: project.id, name: name.into(), folder: folder.into(), description: String::new(), position: 0, created_at: now, updated_at: now }
}

fn task_of(project: &project::Model) -> task::Model {
    task::Model::new("report".into(), "Report".into(), project.id, Instant::now())
}

#[test]
fn the_project_folder_comes_first_named_after_its_folder() {
    let project = project_in(Some("/work/rekall-rust"));

    let targets = RepositoryTarget::all_of(&task_of(&project), &project, &[]);

    assert_eq!(targets.len(), 1);
    assert_eq!((targets[0].component_id, targets[0].name.as_str()), (None, "rekall-rust"));
    assert_eq!(targets[0].folder.as_deref(), Some(Path::new("/work/rekall-rust")));
}

#[test]
fn a_project_with_no_folder_is_named_by_its_title_and_has_nowhere_to_read() {
    let project = project_in(None);

    let targets = RepositoryTarget::all_of(&task_of(&project), &project, &[]);

    assert_eq!((targets[0].name.as_str(), targets[0].folder.clone()), ("Vega", None));
}

#[test]
fn the_primary_repository_reads_the_task_worktree_while_it_has_one() {
    let project = project_in(Some("/work/rekall-rust"));
    let mut task = task_of(&project);
    task.worktree_folder = Some("/worktrees/rv-168".into());

    let targets = RepositoryTarget::all_of(&task, &project, &[]);

    assert_eq!(targets[0].folder.as_deref(), Some(Path::new("/worktrees/rv-168")));
    assert_eq!(targets[0].name, "rekall-rust");
}

#[test]
fn each_component_is_a_repository_after_the_project_folder_in_the_order_given() {
    let project = project_in(Some("/work/rekall-rust"));
    let site = component("astro-starter", "/work/astro-starter", &project);
    let api = component("api", "/work/api", &project);

    let targets = RepositoryTarget::all_of(&task_of(&project), &project, &[site.clone(), api.clone()]);

    let names: Vec<_> = targets.iter().map(|target| target.name.as_str()).collect();
    assert_eq!(names, ["rekall-rust", "astro-starter", "api"]);
    assert_eq!(targets[1].component_id, Some(site.id));
    assert!(targets[1].reads_own_working_tree);
}

#[test]
fn a_component_inside_the_project_folder_does_not_read_a_working_tree_of_its_own() {
    let project = project_in(Some("/work/rekall-rust"));
    let ui = component("ui", "/work/rekall-rust/rekall-ui", &project);

    let targets = RepositoryTarget::all_of(&task_of(&project), &project, &[ui]);

    assert!(!targets[1].reads_own_working_tree);
}
