//! `TokenUsageService` against a real database: what a session's spend resolves to, how a second
//! read of the same transcript replaces the first, and what the console is told.

mod support;

use rekall_store::TaskStepState;
use rekall_service::token::{TokenCount, UnitUsage, WorkUnit};
use rekall_service::token::TokenUsageStreamEvent;
use support::world;

fn spent(project: Option<&str>, task: &str, step: Option<&str>, output: u64) -> UnitUsage {
    UnitUsage {
        unit: WorkUnit::task(project, task).on_step(step),
        at: None,
        tokens: TokenCount { input: 10, output, cache_creation: 100, cache_read: 1000 },
    }
}

#[tokio::test]
async fn a_session_is_recorded_per_step_and_on_the_task_outside_its_steps() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    let first = world.step(&task, "Draft the schema", 0, TaskStepState::Open).await;
    let second = world.step(&task, "Wire the export", 1, TaskStepState::Open).await;

    let session = [
        spent(Some("vega"), "report-builder", None, 5),
        spent(Some("vega"), "report-builder", Some("1"), 7),
        spent(Some("vega"), "report-builder", Some("Wire the export"), 11),
        spent(Some("vega"), "report-builder", Some("2"), 13),
    ];
    world.services.token_usage.record_session("s-1", 4096, &session).await.unwrap();

    let tallies = world.services.token_usage.tallies().await.unwrap();
    let on = |step: Option<rekall_common::Id>| tallies.iter().find(|t| t.step_id == step).unwrap().output_tokens;
    assert_eq!(tallies.len(), 3);
    assert_eq!(on(None), 5);
    assert_eq!(on(Some(first.id)), 7);
    assert_eq!(on(Some(second.id)), 24);
    assert!(tallies.iter().all(|t| t.task_id == task.id && t.input_tokens > 0 && t.cache_read_tokens > 0));
    assert_eq!(world.services.token_usage.scanned_bytes("s-1").await.unwrap(), Some(4096));
}

#[tokio::test]
async fn reading_a_session_again_replaces_what_it_recorded_instead_of_adding_to_it() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    world.task(&project, "report-builder").await;

    world.services.token_usage.record_session("s-1", 10, &[spent(None, "report-builder", None, 5)]).await.unwrap();
    world.services.token_usage.record_session("s-1", 20, &[spent(None, "report-builder", None, 9)]).await.unwrap();
    world.services.token_usage.record_session("s-2", 30, &[spent(None, "report-builder", None, 1)]).await.unwrap();

    let tallies = world.services.token_usage.tallies().await.unwrap();
    assert_eq!(tallies.len(), 1);
    assert_eq!(tallies[0].output_tokens, 10);
    assert_eq!(world.services.token_usage.scanned_bytes("s-1").await.unwrap(), Some(20));
}

#[tokio::test]
async fn a_task_that_does_not_resolve_is_dropped_and_a_step_that_does_not_falls_back_to_its_task() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    world.task(&project, "report-builder").await;

    let session = [spent(Some("vega"), "nowhere", None, 5), spent(Some("vega"), "report-builder", Some("9"), 7)];
    world.services.token_usage.record_session("s-1", 10, &session).await.unwrap();

    let tallies = world.services.token_usage.tallies().await.unwrap();
    assert_eq!(tallies.len(), 1);
    assert_eq!((tallies[0].step_id, tallies[0].output_tokens), (None, 7));
}

#[tokio::test]
async fn a_change_is_announced_for_each_task_it_touches_and_an_unchanged_read_announces_nothing() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let report = world.task(&project, "report-builder").await;
    let export = world.task(&project, "export").await;
    let mut feed = world.services.ctx.events.subscribe();

    world.services.token_usage.record_session("s-1", 10, &[spent(None, "report-builder", None, 5)]).await.unwrap();
    world.services.token_usage.record_session("s-1", 20, &[spent(None, "export", None, 5)]).await.unwrap();
    world.services.token_usage.record_session("s-1", 30, &[spent(None, "export", None, 5)]).await.unwrap();

    let mut announced = Vec::new();
    while let Ok(event) = feed.try_recv() {
        if let Some(event) = event.downcast::<TokenUsageStreamEvent>() {
            announced.push((event.task_id, event.tallies.len()));
        }
    }
    assert_eq!(announced.len(), 3);
    assert_eq!(announced[0], (report.id, 1));
    assert!(announced[1..].contains(&(report.id, 0)));
    assert!(announced[1..].contains(&(export.id, 1)));
}

#[tokio::test]
async fn deleting_a_step_leaves_its_tokens_on_the_task() {
    use sea_orm::EntityTrait;
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Draft", 0, TaskStepState::Open).await;
    world.services.token_usage.record_session("s-1", 10, &[spent(None, "report-builder", Some("1"), 5)]).await.unwrap();

    rekall_store::prelude::TaskStep::delete_by_id(step.id).exec(world.db()).await.unwrap();

    let tallies = world.services.token_usage.tallies().await.unwrap();
    assert_eq!((tallies[0].task_id, tallies[0].step_id, tallies[0].output_tokens), (task.id, None, 5));
}

#[tokio::test]
async fn only_the_folders_set_on_projects_are_watched_once_each() {
    let world = world().await;
    world.project("vega", Some("/work/vega"), false).await;
    world.project("lyra", Some(" /work/vega "), false).await;
    world.project("orion", None, false).await;

    let folders = world.services.token_usage.watched_folders().await.unwrap();

    assert_eq!(folders, [std::path::PathBuf::from("/work/vega")]);
}

fn spent_at(task: &str, at: &str, output: u64) -> UnitUsage {
    UnitUsage { at: Some(at.parse().unwrap()), ..spent(None, task, None, output) }
}

#[tokio::test]
async fn a_tasks_spend_is_told_day_by_day_oldest_first_across_its_steps() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    world.step(&task, "Draft", 0, TaskStepState::Open).await;

    let session = [
        spent_at("report-builder", "2026-10-02T09:00:00Z", 4),
        UnitUsage { unit: WorkUnit::task(None, "report-builder").on_step(Some("1")), ..spent_at("report-builder", "2026-10-02T23:30:00Z", 6) },
        spent_at("report-builder", "2026-10-01T22:00:00Z", 3),
    ];
    world.services.token_usage.record_session("s-1", 10, &session).await.unwrap();

    let days = world.services.token_usage.days().await.unwrap();
    let shown: Vec<(String, i64)> = days.iter().map(|d| (d.day.to_string(), d.output_tokens)).collect();
    assert_eq!(shown, [("2026-10-01T00:00:00Z".to_string(), 3), ("2026-10-02T00:00:00Z".to_string(), 10)]);
    assert!(days.iter().all(|d| d.task_id == task.id));
}

#[tokio::test]
async fn a_read_that_finds_the_same_tokens_on_another_day_replaces_the_rows_and_announces_it() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    world.task(&project, "report-builder").await;
    world.services.token_usage.record_session("s-1", 10, &[spent_at("report-builder", "2026-10-01T10:00:00Z", 5)]).await.unwrap();
    let mut feed = world.services.ctx.events.subscribe();

    world.services.token_usage.record_session("s-1", 20, &[spent_at("report-builder", "2026-10-03T10:00:00Z", 5)]).await.unwrap();

    let days = world.services.token_usage.days().await.unwrap();
    assert_eq!(days.len(), 1);
    assert_eq!(days[0].day.to_string(), "2026-10-03T00:00:00Z");
    assert!(matches!(feed.try_recv(), Ok(event) if event.downcast::<TokenUsageStreamEvent>().is_some_and(|event| event.days.len() == 1)));
}
