use rekall_core::connection::{InMemorySecretStore, ScriptedGitlabNamespaces, Secret, TrackerFailure};
use rekall_core::EventBus;
use rekall_kernel::Instant;
use rekall_store::external_connection;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

struct World {
    service: GitlabTargetService,
    connection: external_connection::Model,
}

async fn world(gitlab: ScriptedGitlabNamespaces, credential_stored: bool, root_path: Option<&str>) -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let mut connection = external_connection::Model::new("acme".into(), "Acme GitLab".into(), ConnectionKind::Gitlab, Some("https://gitlab.acme.dev".into()), Instant::now());
    connection.secret_set_at = credential_stored.then(Instant::now);
    connection.root_path = root_path.map(str::to_string);
    connection.clone().into_active_model().insert(&database.conn).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    secrets.put(&connection.secret_ref, &Secret::new("glpat-secret")).unwrap();
    World { service: GitlabTargetService::new(ctx, secrets, Arc::new(gitlab)), connection }
}

fn acme() -> ScriptedGitlabNamespaces {
    ScriptedGitlabNamespaces::new().with_group("platform").with_group("platform/backend").with_project("platform/backend/payments-api").with_project("platform/backend/ledger-service")
}

#[tokio::test]
async fn a_project_gitlab_has_is_found_from_a_pasted_address() {
    let world = world(acme(), true, Some("platform")).await;

    let checked = world.service.check(world.connection.id, "https://gitlab.acme.dev/platform/backend/payments-api.git").await.unwrap();

    assert!(matches!(checked, TargetCheckView::Found(found) if found.path == "platform/backend/payments-api"));
}

#[tokio::test]
async fn a_misspelt_path_is_not_found_and_names_the_nearest_sibling() {
    let world = world(acme(), true, Some("platform")).await;

    let checked = world.service.check(world.connection.id, "platform/backend/paymnts-api").await.unwrap();

    assert_eq!(checked, TargetCheckView::NotFound { nearest: Some("platform/backend/payments-api".into()) });
}

#[tokio::test]
async fn a_path_outside_the_root_path_is_not_asked_of_gitlab() {
    let world = world(ScriptedGitlabNamespaces::failing(TrackerFailure::Failed { reason: "must not be asked".into() }), true, Some("platform")).await;

    let checked = world.service.check(world.connection.id, "other-team/billing").await.unwrap();

    assert_eq!(checked, TargetCheckView::OutsideRoot { root: "platform".into() });
}

#[tokio::test]
async fn a_connection_without_a_credential_cannot_check_a_path() {
    let world = world(acme(), false, None).await;

    assert_eq!(world.service.check(world.connection.id, "platform").await.unwrap(), TargetCheckView::NoCredential);
}

#[tokio::test]
async fn a_gitlab_that_does_not_answer_is_unreachable_with_its_reason() {
    let world = world(ScriptedGitlabNamespaces::failing(TrackerFailure::Failed { reason: "timed out".into() }), true, None).await;

    let checked = world.service.check(world.connection.id, "platform").await.unwrap();

    assert_eq!(checked, TargetCheckView::Unreachable { reason: "GitLab could not be asked: timed out.".into() });
}

#[tokio::test]
async fn the_children_of_a_group_are_listed_and_default_to_the_root_path() {
    let world = world(acme(), true, Some("platform/backend")).await;

    let children = world.service.children(world.connection.id, None, None).await.unwrap();

    assert_eq!(children.iter().map(|child| child.path.as_str()).collect::<Vec<_>>(), ["platform/backend/payments-api", "platform/backend/ledger-service"]);
}

#[tokio::test]
async fn the_children_of_a_group_outside_the_root_path_are_refused() {
    let world = world(acme(), true, Some("platform/backend")).await;

    let error = world.service.children(world.connection.id, Some("platform"), None).await.unwrap_err();

    assert!(error.is_illegal_argument());
}

#[tokio::test]
async fn a_target_gitlab_lacks_is_refused_and_one_it_has_passes() {
    let world = world(acme(), true, None).await;

    assert!(world.service.require_existing(&world.connection, "platform/backend/payments-api").await.is_ok());
    let error = world.service.require_existing(&world.connection, "platform/backend/nope").await.unwrap_err();
    assert!(error.is_illegal_argument());
    assert!(error.message().contains("GitLab has no project or group 'platform/backend/nope'"));
}

#[tokio::test]
async fn a_target_is_not_checked_when_no_credential_is_stored() {
    let world = world(ScriptedGitlabNamespaces::new(), false, None).await;

    assert!(world.service.require_existing(&world.connection, "anything/at/all").await.is_ok());
}
