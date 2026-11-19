use rekall_kernel::Instant;
use rekall_store::{company, project, ProjectStatus};
use rekall_core::connection::InMemorySecretStore;
use rekall_core::EventBus;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;
use crate::connection::{ProjectConnectionCommand, ProjectConnectionService};

struct World {
    connections: ConnectionService,
    bindings: ProjectConnectionService,
    acme: company::Model,
    globex: company::Model,
    vega: project::Model,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let acme = a_company(&ctx, "Acme", now).await;
    let globex = a_company(&ctx, "Globex", now).await;
    let vega = project::Model {
        id: Id::random(),
        label: "vega".into(),
        title: "Vega".into(),
        status: ProjectStatus::Active,
        icon: "folder".into(),
        description: None,
        blueprint_markdown: None,
        repo_folder: None,
        auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(),
        company_id: acme.id,
        created_at: now,
        updated_at: now,
    };
    vega.clone().into_active_model().insert(&ctx.db).await.unwrap();
    World {
        connections: ConnectionService::new(ctx.clone(), Arc::new(InMemorySecretStore::new())),
        bindings: ProjectConnectionService::new(ctx),
        acme,
        globex,
        vega,
    }
}

async fn a_company(ctx: &Ctx, name: &str, now: Instant) -> company::Model {
    let company = company::Model { id: Id::random(), name: name.into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&ctx.db).await.unwrap();
    company
}

fn gitlab(title: &str, company_ids: Vec<Id>) -> ConnectionCommand {
    ConnectionCommand {
        kind: Some("GITLAB".into()),
        title: Some(title.into()),
        base_url: Some("https://gitlab.example.com".into()),
        root_path: Some("/acme/".into()),
        company_ids: Some(company_ids),
        ..ConnectionCommand::default()
    }
}

fn target(path: &str) -> ProjectConnectionCommand {
    ProjectConnectionCommand { label: None, target: Some(path.into()), task_manager: None }
}

#[tokio::test]
async fn a_connection_is_created_for_the_workspace_shared_with_the_companies_named_and_keeps_a_clean_root_path() {
    let world = world().await;

    let created = world.connections.create(gitlab("GitLab", vec![world.acme.id, world.globex.id])).await.unwrap();

    assert_eq!(created.root_path.as_deref(), Some("acme"));
    assert_eq!(created.company_ids.len(), 2);
    assert_eq!(world.connections.list_of_company(world.globex.id).await.unwrap().len(), 1);
    assert_eq!(world.connections.list_all().await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_connection_shared_with_nobody_is_listed_for_the_workspace_and_for_no_company() {
    let world = world().await;

    world.connections.create(gitlab("GitLab", vec![])).await.unwrap();

    assert_eq!(world.connections.list_all().await.unwrap().len(), 1);
    assert!(world.connections.list_of_company(world.acme.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_label_is_taken_for_the_whole_workspace_whatever_company_holds_it() {
    let world = world().await;
    world.connections.create(gitlab("GitLab", vec![world.acme.id])).await.unwrap();

    let refused = world.connections.create(gitlab("GitLab", vec![world.globex.id])).await.unwrap_err();

    assert!(matches!(refused, RekallError::Conflict(_)), "{refused:?}");
}

#[tokio::test]
async fn a_company_that_does_not_exist_is_refused_and_nothing_is_created() {
    let world = world().await;

    let refused = world.connections.create(gitlab("GitLab", vec![Id::random()])).await.unwrap_err();

    assert!(matches!(refused, RekallError::NotFound(_)), "{refused:?}");
    assert!(world.connections.list_all().await.unwrap().is_empty());
}

#[tokio::test]
async fn an_edit_never_changes_the_companies_a_connection_is_shared_with() {
    let world = world().await;
    let created = world.connections.create(gitlab("GitLab", vec![world.acme.id])).await.unwrap();
    let edit = ConnectionCommand { title: Some("Acme GitLab".into()), company_ids: Some(vec![world.globex.id]), ..ConnectionCommand::default() };

    let edited = world.connections.update(created.id, edit).await.unwrap();

    assert_eq!(edited.title, "Acme GitLab");
    assert_eq!(edited.company_ids, [world.acme.id]);
}

#[tokio::test]
async fn a_grafana_connection_accepts_a_self_signed_certificate_until_an_edit_turns_it_off_and_other_kinds_refuse_it() {
    let world = world().await;
    let grafana = ConnectionCommand {
        kind: Some("GRAFANA".into()),
        title: Some("Grafana".into()),
        base_url: Some("https://10.36.97.212/grafana".into()),
        accept_self_signed: Some(true),
        ..ConnectionCommand::default()
    };
    let created = world.connections.create(grafana).await.unwrap();
    assert!(created.accept_self_signed);

    let untouched = world.connections.update(created.id, ConnectionCommand { title: Some("Ops Grafana".into()), ..ConnectionCommand::default() }).await.unwrap();
    assert!(untouched.accept_self_signed);

    let off = world.connections.update(created.id, ConnectionCommand { accept_self_signed: Some(false), ..ConnectionCommand::default() }).await.unwrap();
    assert!(!off.accept_self_signed);

    let gitlab = ConnectionCommand { accept_self_signed: Some(true), ..gitlab("GitLab", Vec::new()) };
    assert!(world.connections.create(gitlab).await.is_err());
}

#[tokio::test]
async fn sharing_replaces_the_companies_and_the_detail_lists_them() {
    let world = world().await;
    let created = world.connections.create(gitlab("GitLab", vec![world.acme.id])).await.unwrap();

    world.connections.set_companies(created.id, vec![world.globex.id]).await.unwrap();

    let detail = world.connections.get(created.id).await.unwrap();
    assert_eq!(detail.companies.iter().map(|company| company.name.as_str()).collect::<Vec<_>>(), ["Globex"]);
}

#[tokio::test]
async fn a_company_cannot_be_removed_while_one_of_its_projects_binds_the_connection() {
    let world = world().await;
    let created = world.connections.create(gitlab("GitLab", vec![world.acme.id, world.globex.id])).await.unwrap();
    let binding = world.bindings.bind(world.vega.id, created.id, target("acme/vega")).await.unwrap();

    let refused = world.connections.set_companies(created.id, vec![world.globex.id]).await.unwrap_err();

    assert!(matches!(&refused, RekallError::Conflict(message) if message.starts_with("1 of the projects")), "{refused:?}");
    assert_eq!(world.connections.get(created.id).await.unwrap().connection.company_ids.len(), 2);

    world.bindings.unbind(world.vega.id, binding.id).await.unwrap();
    world.connections.set_companies(created.id, vec![world.globex.id]).await.unwrap();
    assert_eq!(world.connections.get(created.id).await.unwrap().connection.company_ids, [world.globex.id]);
}

#[tokio::test]
async fn a_company_can_be_added_while_projects_bind_the_connection() {
    let world = world().await;
    let created = world.connections.create(gitlab("GitLab", vec![world.acme.id])).await.unwrap();
    world.bindings.bind(world.vega.id, created.id, target("acme/vega")).await.unwrap();

    let shared = world.connections.set_companies(created.id, vec![world.acme.id, world.globex.id]).await.unwrap();

    assert_eq!(shared.company_ids.len(), 2);
}

#[tokio::test]
async fn a_project_binds_only_a_connection_shared_with_its_company() {
    let world = world().await;
    let created = world.connections.create(gitlab("GitLab", vec![world.globex.id])).await.unwrap();

    let refused = world.bindings.bind(world.vega.id, created.id, target("acme/vega")).await.unwrap_err();

    assert!(matches!(&refused, RekallError::IllegalArgument(message) if message.contains("Share it with that company first")), "{refused:?}");
    world.connections.set_companies(created.id, vec![world.globex.id, world.acme.id]).await.unwrap();
    assert!(world.bindings.bind(world.vega.id, created.id, target("acme/vega")).await.is_ok());
}

#[tokio::test]
async fn deleting_a_connection_reports_the_projects_that_lose_it() {
    let world = world().await;
    let created = world.connections.create(gitlab("GitLab", vec![world.acme.id])).await.unwrap();
    world.bindings.bind(world.vega.id, created.id, target("acme/vega")).await.unwrap();

    let removed = world.connections.delete(created.id).await.unwrap();

    assert_eq!(removed.unbound_projects, 1);
    assert!(world.connections.list_all().await.unwrap().is_empty());
}

fn as_task_manager(command: ProjectConnectionCommand) -> ProjectConnectionCommand {
    ProjectConnectionCommand { task_manager: Some(true), ..command }
}

#[tokio::test]
async fn a_project_has_one_task_manager_and_choosing_another_binding_releases_the_first() {
    let world = world().await;
    let gitlab = world.connections.create(gitlab("GitLab", vec![world.acme.id])).await.unwrap();
    let github = ConnectionCommand {
        kind: Some("GITHUB".into()),
        title: Some("GitHub".into()),
        base_url: Some("https://github.com".into()),
        company_ids: Some(vec![world.acme.id]),
        ..ConnectionCommand::default()
    };
    let github = world.connections.create(github).await.unwrap();
    let first = world.bindings.bind(world.vega.id, gitlab.id, target("acme/vega")).await.unwrap();
    let second = world.bindings.bind(world.vega.id, github.id, target("acme/vega")).await.unwrap();

    world.bindings.update(world.vega.id, first.id, as_task_manager(target("acme/vega"))).await.unwrap();
    world.bindings.update(world.vega.id, second.id, as_task_manager(target("acme/vega"))).await.unwrap();

    let managers: Vec<String> = world.bindings.list(world.vega.id).await.unwrap().into_iter().filter(|binding| binding.task_manager).map(|binding| binding.label).collect();
    assert_eq!(managers, std::slice::from_ref(&second.label));
    assert_eq!(world.bindings.task_manager(world.vega.id).await.unwrap().unwrap().0.id, second.id);
}

#[tokio::test]
async fn a_task_manager_has_to_name_where_the_issues_go_and_a_cluster_is_none() {
    let world = world().await;
    let gitlab = world.connections.create(gitlab("GitLab", vec![world.acme.id])).await.unwrap();
    let grafana_command = ConnectionCommand { kind: Some("GRAFANA".into()), title: Some("Grafana".into()), base_url: Some("https://grafana.example.com".into()), company_ids: Some(vec![world.acme.id]), ..ConnectionCommand::default() };
    let grafana = world.connections.create(grafana_command).await.unwrap();
    let without_target = world.bindings.bind(world.vega.id, gitlab.id, ProjectConnectionCommand::default()).await.unwrap();
    let on_grafana = world.bindings.bind(world.vega.id, grafana.id, target("dash")).await.unwrap();

    let no_target = world.bindings.update(world.vega.id, without_target.id, as_task_manager(ProjectConnectionCommand::default())).await.unwrap_err();
    let no_manager = world.bindings.update(world.vega.id, on_grafana.id, as_task_manager(target("dash"))).await.unwrap_err();

    assert!(matches!(no_target, RekallError::IllegalArgument(_)), "{no_target:?}");
    assert!(matches!(no_manager, RekallError::IllegalArgument(_)), "{no_manager:?}");
    assert!(world.bindings.task_manager(world.vega.id).await.unwrap().is_none());
}

#[tokio::test]
async fn editing_a_binding_without_a_choice_leaves_it_the_task_manager_and_unchecking_it_clears_the_choice() {
    let world = world().await;
    let gitlab = world.connections.create(gitlab("GitLab", vec![world.acme.id])).await.unwrap();
    let binding = world.bindings.bind(world.vega.id, gitlab.id, target("acme/vega")).await.unwrap();
    world.bindings.update(world.vega.id, binding.id, as_task_manager(target("acme/vega"))).await.unwrap();

    let renamed = world.bindings.update(world.vega.id, binding.id, target("acme/vega-two")).await.unwrap();
    assert!(renamed.task_manager);

    let released = world.bindings.update(world.vega.id, binding.id, ProjectConnectionCommand { task_manager: Some(false), ..target("acme/vega-two") }).await.unwrap();
    assert!(!released.task_manager);
    assert!(world.bindings.task_manager(world.vega.id).await.unwrap().is_none());
}
