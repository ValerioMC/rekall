use super::*;

#[test]
fn a_blank_main_branch_means_the_folders_current_branch_and_a_name_is_trimmed() {
    assert_eq!(main_branch_of(None).unwrap(), None);
    assert_eq!(main_branch_of(Some("  ")).unwrap(), None);
    assert_eq!(main_branch_of(Some(" develop ")).unwrap().as_deref(), Some("develop"));
    assert_eq!(main_branch_of(Some("release/1.2")).unwrap().as_deref(), Some("release/1.2"));
}

#[test]
fn a_name_git_could_read_as_an_option_or_a_range_is_refused() {
    for name in ["-D", "main..other", "a b", "main~1", "x:y", "feature/", "/feature", "topic.lock", "a@{1}", "a\\b"] {
        assert!(main_branch_of(Some(name)).is_err(), "{name} was accepted");
    }
}

mod pull_request_flow {
    use std::sync::Arc;

    use rekall_core::connection::InMemorySecretStore;
    use rekall_core::EventBus;
    use rekall_store::company;
    use sea_orm::{ActiveModelTrait, IntoActiveModel};

    use super::*;
    use crate::connection::{ConnectionCommand, ConnectionService, ProjectConnectionCommand, ProjectConnectionService};

    struct World {
        projects: ProjectService,
        connections: ConnectionService,
        bindings: ProjectConnectionService,
        company: company::Model,
        project: ProjectView,
    }

    async fn world() -> World {
        let database = rekall_store::open_in_memory().await.unwrap();
        let ctx = Ctx::new(database.conn.clone(), EventBus::new());
        let now = ctx.now();
        let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
        company.clone().into_active_model().insert(&ctx.db).await.unwrap();
        let projects = ProjectService::new(ctx.clone(), GitRepositoryInspector, PullRequestFlowService);
        let project = projects.create(ProjectCommand { label: Some("vega".into()), title: Some("Vega".into()), company_id: Some(company.id), ..ProjectCommand::default() }).await.unwrap();
        World {
            projects,
            connections: ConnectionService::new(ctx.clone(), Arc::new(InMemorySecretStore::new())),
            bindings: ProjectConnectionService::new(ctx),
            company,
            project,
        }
    }

    impl World {
        async fn make_task_manager(&self, kind: &str, target: &str) {
            let command = ConnectionCommand {
                kind: Some(kind.into()),
                title: Some(kind.into()),
                base_url: Some("https://tracker.example.com".into()),
                company_ids: Some(vec![self.company.id]),
                ..ConnectionCommand::default()
            };
            let connection = self.connections.create(command).await.unwrap();
            let binding = self.bindings.bind(self.project.id, connection.id, ProjectConnectionCommand { target: Some(target.into()), ..ProjectConnectionCommand::default() }).await.unwrap();
            self.bindings.update(self.project.id, binding.id, ProjectConnectionCommand { target: Some(target.into()), task_manager: Some(true), ..ProjectConnectionCommand::default() }).await.unwrap();
        }

        fn save(&self, repo_folder: Option<&str>, main_branch: Option<&str>, pull_request_flow: Option<bool>) -> ProjectCommand {
            ProjectCommand {
                label: Some(self.project.label.clone()),
                title: Some(self.project.title.clone()),
                company_id: Some(self.company.id),
                repo_folder: repo_folder.map(String::from),
                main_branch: main_branch.map(String::from),
                pull_request_flow,
                ..ProjectCommand::default()
            }
        }
    }

    #[tokio::test]
    async fn a_new_project_has_the_flow_off() {
        let world = world().await;

        assert!(!world.project.pull_request_flow);
    }

    #[tokio::test]
    async fn the_flow_turns_on_with_a_folder_a_main_branch_and_a_github_task_manager_and_stays_on_when_not_mentioned() {
        let world = world().await;
        world.make_task_manager("GITHUB", "acme/vega").await;

        let on = world.projects.update(world.project.id, world.save(Some("/work/vega"), Some("main"), Some(true))).await.unwrap();
        let untouched = world.projects.update(world.project.id, world.save(Some("/work/vega"), Some("main"), None)).await.unwrap();
        let off = world.projects.update(world.project.id, world.save(Some("/work/vega"), Some("main"), Some(false))).await.unwrap();

        assert!(on.pull_request_flow);
        assert!(untouched.pull_request_flow);
        assert!(!off.pull_request_flow);
        assert!(!world.projects.get(world.project.id).await.unwrap().pull_request_flow);
    }

    #[tokio::test]
    async fn the_flow_is_refused_without_a_folder_a_branch_or_a_task_manager_and_stays_off() {
        let world = world().await;
        let without_manager = world.projects.update(world.project.id, world.save(Some("/work/vega"), Some("main"), Some(true))).await.unwrap_err();
        world.make_task_manager("GITHUB", "acme/vega").await;
        let without_folder = world.projects.update(world.project.id, world.save(None, Some("main"), Some(true))).await.unwrap_err();
        let without_branch = world.projects.update(world.project.id, world.save(Some("/work/not-a-repository"), None, Some(true))).await.unwrap_err();

        for refusal in [&without_manager, &without_folder, &without_branch] {
            assert!(matches!(refusal, RekallError::IllegalArgument(_)), "{refusal:?}");
        }
        assert!(without_manager.to_string().contains("task manager"), "{without_manager}");
        assert!(without_folder.to_string().contains("folder"), "{without_folder}");
        assert!(without_branch.to_string().contains("branch"), "{without_branch}");
        assert!(!world.projects.get(world.project.id).await.unwrap().pull_request_flow);
    }

    #[tokio::test]
    async fn the_flow_is_refused_for_a_jira_task_manager() {
        let world = world().await;
        world.make_task_manager("JIRA", "VEGA").await;

        let refusal = world.projects.update(world.project.id, world.save(Some("/work/vega"), Some("main"), Some(true))).await.unwrap_err();

        assert!(matches!(refusal, RekallError::IllegalArgument(_)), "{refusal:?}");
    }

    #[tokio::test]
    async fn a_task_manager_cannot_become_jira_be_unchecked_or_be_unbound_while_the_flow_is_on() {
        let world = world().await;
        world.make_task_manager("GITHUB", "acme/vega").await;
        world.projects.update(world.project.id, world.save(Some("/work/vega"), Some("main"), Some(true))).await.unwrap();
        let (github, _) = world.bindings.task_manager(world.project.id).await.unwrap().unwrap();
        let jira = world.connections.create(ConnectionCommand { kind: Some("JIRA".into()), title: Some("Jira".into()), base_url: Some("https://jira.example.com".into()), company_ids: Some(vec![world.company.id]), ..ConnectionCommand::default() }).await.unwrap();
        let jira_binding = world.bindings.bind(world.project.id, jira.id, ProjectConnectionCommand { target: Some("VEGA".into()), ..ProjectConnectionCommand::default() }).await.unwrap();

        let to_jira = world.bindings.update(world.project.id, jira_binding.id, ProjectConnectionCommand { target: Some("VEGA".into()), task_manager: Some(true), ..ProjectConnectionCommand::default() }).await.unwrap_err();
        let unchecked = world.bindings.update(world.project.id, github.id, ProjectConnectionCommand { target: Some("acme/vega".into()), task_manager: Some(false), ..ProjectConnectionCommand::default() }).await.unwrap_err();
        let unbound = world.bindings.unbind(world.project.id, github.id).await.unwrap_err();

        for refusal in [&to_jira, &unchecked, &unbound] {
            assert!(matches!(refusal, RekallError::IllegalArgument(_)), "{refusal:?}");
        }
        assert_eq!(world.bindings.task_manager(world.project.id).await.unwrap().unwrap().0.id, github.id);
    }

    #[tokio::test]
    async fn with_the_flow_off_the_task_manager_can_change_and_go() {
        let world = world().await;
        world.make_task_manager("GITHUB", "acme/vega").await;
        let (github, _) = world.bindings.task_manager(world.project.id).await.unwrap().unwrap();

        world.bindings.unbind(world.project.id, github.id).await.unwrap();

        assert!(world.bindings.task_manager(world.project.id).await.unwrap().is_none());
    }
}
