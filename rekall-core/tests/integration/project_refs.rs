//! A project label is unique within a company only: a bare label two companies share is refused
//! with the company of each candidate, and a company beside it picks one.

mod support;

use rekall_core::flow::FlowPlan;
use rekall_core::task::TaskBrief;
use rekall_core::ProjectRef;
use rekall_kernel::RekallError;
use rekall_store::repository as repo;
use support::world;

fn brief(title: &str) -> TaskBrief {
    TaskBrief { title: Some(title.into()), description: None, kind: rekall_core::task::TaskKind::Task, epic: None }
}

async fn two_companies_sharing_a_label() -> (support::World, rekall_store::project::Model, rekall_store::project::Model) {
    let world = world().await;
    let acme = world.company("Acme Robotics").await;
    let forge = world.company("VFORGE").await;
    let in_acme = world.project_in(&acme, "vega", None, false).await;
    let in_forge = world.project_in(&forge, "vega", None, false).await;
    (world, in_acme, in_forge)
}

#[tokio::test]
async fn a_bare_label_two_companies_share_is_refused_and_names_each_company() {
    let (world, _, _) = two_companies_sharing_a_label().await;

    let refused = world.services.task_authoring.create(&ProjectRef::bare("vega"), &brief("A task"), &FlowPlan::new(Vec::new())).await.unwrap_err();

    match refused {
        RekallError::AmbiguousAnchor { candidates, .. } => {
            assert_eq!(candidates, vec!["company:Acme Robotics project:vega", "company:VFORGE project:vega"]);
        }
        other => panic!("expected an ambiguous anchor, got {other:?}"),
    }
}

#[tokio::test]
async fn a_company_beside_the_label_creates_the_task_in_that_companys_project() {
    let (world, in_acme, in_forge) = two_companies_sharing_a_label().await;

    world.services.task_authoring.create(&ProjectRef::in_company("vforge", "vega"), &brief("A task"), &FlowPlan::new(Vec::new())).await.unwrap();

    assert_eq!(repo::task::list_of_project(world.db(), in_forge.id).await.unwrap().len(), 1);
    assert!(repo::task::list_of_project(world.db(), in_acme.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_company_without_that_project_is_an_unknown_anchor() {
    let (world, _, _) = two_companies_sharing_a_label().await;
    world.company("Empty Co").await;

    let refused = world.services.task_authoring.create(&ProjectRef::in_company("Empty Co", "vega"), &brief("A task"), &FlowPlan::new(Vec::new())).await.unwrap_err();

    assert!(matches!(refused, RekallError::UnknownAnchor(message) if message.contains("company:Empty Co project:vega")));
}
