//! A session replacing a task's description: only the description changes, and the write is not a revision.

mod support;

use rekall_store::repository as repo;
use rekall_store::task::DESCRIPTION_MAX_CHARACTERS;
use rekall_store::{task, TaskStepState};
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use support::world;

async fn described(world: &support::World, project: &rekall_store::project::Model, label: &str, text: &str) -> task::Model {
    let mut task = world.task(project, label).await;
    task.description = Some(text.into());
    task.clone().into_active_model().reset_all().update(world.db()).await.unwrap();
    task
}

#[tokio::test]
async fn replacing_overwrites_the_description_and_changes_nothing_else() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let before = described(&world, &project, "checkout", "pay thing wrong totals").await;

    let replaced = world.services.task_descriptions.replace(Some("vega"), "checkout", Some("  ## Problem\n\nThe totals are wrong.  ")).await.unwrap();

    assert_eq!((replaced.task_anchor.as_str(), replaced.changed), ("task:CHECKOUT", true));
    let after = repo::task::require(world.db(), before.id).await.unwrap();
    assert_eq!(after.description.as_deref(), Some("## Problem\n\nThe totals are wrong."));
    assert_eq!((after.title, after.label, after.status), (before.title, before.label, before.status));
}

#[tokio::test]
async fn writing_the_text_it_already_has_reports_no_change() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    described(&world, &project, "checkout", "The totals are wrong.").await;

    let replaced = world.services.task_descriptions.replace(Some("vega"), "checkout", Some("The totals are wrong.")).await.unwrap();

    assert!(!replaced.changed);
}

#[tokio::test]
async fn a_blank_or_oversized_body_is_refused_and_the_description_stays() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let before = described(&world, &project, "checkout", "The totals are wrong.").await;

    let blank = world.services.task_descriptions.replace(Some("vega"), "checkout", Some("   ")).await.unwrap_err();
    let missing = world.services.task_descriptions.replace(Some("vega"), "checkout", None).await.unwrap_err();
    let long = world.services.task_descriptions.replace(Some("vega"), "checkout", Some(&"x".repeat(DESCRIPTION_MAX_CHARACTERS + 1))).await.unwrap_err();

    assert!(blank.is_illegal_argument() && missing.is_illegal_argument());
    assert!(long.message().contains(&format!("capped at {DESCRIPTION_MAX_CHARACTERS} characters and this one is {}", DESCRIPTION_MAX_CHARACTERS + 1)), "{long:?}");
    assert_eq!(repo::task::require(world.db(), before.id).await.unwrap().description, before.description);
}

#[tokio::test]
async fn a_task_that_was_worked_gets_no_revision_from_a_rewrite() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = described(&world, &project, "checkout", "pay thing wrong totals").await;
    world.step(&task, "Fix the totals", 1, TaskStepState::Claimed).await;

    world.services.task_descriptions.replace(Some("vega"), "checkout", Some("The totals are wrong.")).await.unwrap();

    assert!(repo::task_feedback::list_of_task(world.db(), task.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn an_unknown_task_is_not_found() {
    let world = world().await;
    world.project("vega", None, false).await;

    let refused = world.services.task_descriptions.replace(Some("vega"), "nothing", Some("text")).await.unwrap_err();

    assert!(matches!(refused, rekall_kernel::RekallError::UnknownAnchor(_)), "{refused:?}");
}
