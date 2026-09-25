//! The PTY terminal manager against a stand-in `claude`: launch arguments, the running marker,
//! input and output, the exit path, the cap, and the refusals.

mod support;

use std::sync::Arc;

use rekall_common::RekallError;
use rekall_model::TaskStepState;
use rekall_claude::terminal::TerminalMode;
use support::{eventually, world, Recorder};

#[tokio::test]
async fn a_terminal_starts_claude_in_the_project_folder_with_rk_typed_and_echoes_input() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Open).await;
    let terminals = world.terminals(8);
    let mut ended = terminals.subscribe_ended();

    let view = terminals.open(task.id, Some(step.id), true, Some("opus"), Some("bogus"), TerminalMode::Work).await.unwrap();
    assert_eq!(view.task_id, task.id);
    assert_eq!(view.step_id, Some(step.id));
    assert_eq!(view.anchors, "project:alpha task:one");
    assert!(view.skip_permissions);
    assert_eq!(view.model.as_deref(), Some("opus"));
    assert_eq!(view.effort, None, "an effort level the settings do not offer is dropped");
    assert!(view.live);
    assert_eq!(world.step_state(&task, &step).await, TaskStepState::Running);

    let recorder = Arc::new(Recorder::default());
    let (_, token) = terminals.attach(view.id, recorder.clone()).unwrap();
    eventually("the launch arguments", || async {
        recorder.text().contains("ARGS:--dangerously-skip-permissions --model opus /rk project:alpha task:one")
    })
    .await;

    terminals.write(view.id, b"hello\n").unwrap();
    eventually("the echoed line", || async { recorder.text().contains("GOT:hello") }).await;

    // A second open on the same task is routed to the live terminal.
    let again = terminals.open(task.id, None, false, None, None, TerminalMode::Work).await.unwrap();
    assert_eq!(again.id, view.id);
    assert_eq!(terminals.live_count(), 1);

    terminals.write(view.id, b"exit\n").unwrap();
    let end = tokio::time::timeout(std::time::Duration::from_secs(5), ended.recv()).await.unwrap().unwrap();
    assert_eq!(end.terminal_id, view.id);
    assert_eq!(end.task_id, task.id);
    eventually("the end notice", || async { recorder.ended.lock().unwrap().is_some() }).await;
    assert_eq!(*recorder.ended.lock().unwrap(), Some((3, "claude exited with code 3".to_string())));
    assert_eq!(terminals.live_count(), 0);
    assert_eq!(world.step_state(&task, &step).await, TaskStepState::Open, "the running marker goes back");
    terminals.detach(view.id, token);

    assert!(matches!(terminals.write(view.id, b"late\n"), Err(RekallError::Conflict(_))));
}

#[tokio::test]
async fn a_plan_terminal_opens_on_the_task_with_the_plan_line_and_a_plan_on_a_live_one_is_typed_into_it() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Open).await;
    let terminals = world.terminals(8);

    let view = terminals.open(task.id, Some(step.id), false, None, None, TerminalMode::Plan).await.unwrap();
    assert_eq!(view.step_id, None, "a plan is on the task, never on a step");
    assert_eq!(world.step_state(&task, &step).await, TaskStepState::Open);

    let recorder = Arc::new(Recorder::default());
    let (_, token) = terminals.attach(view.id, recorder.clone()).unwrap();
    eventually("the plan line", || async { recorder.text().contains("ARGS:/rk project:alpha task:one plan") }).await;
    terminals.detach(view.id, token);
    terminals.close(view.id, "done").await;

    let working = terminals.open(task.id, None, false, None, None, TerminalMode::Work).await.unwrap();
    let recorder = Arc::new(Recorder::default());
    let (_, token) = terminals.attach(working.id, recorder.clone()).unwrap();
    eventually("the work line", || async { recorder.text().contains("ARGS:/rk project:alpha task:one") }).await;

    let planned = terminals.open(task.id, Some(step.id), false, None, None, TerminalMode::Plan).await.unwrap();
    assert_eq!(planned.id, working.id, "a plan on a task with a live session reuses it");
    assert_eq!(planned.step_id, None);
    eventually("the typed plan line", || async { recorder.text().contains("GOT:/rk project:alpha task:one plan") }).await;
    assert_eq!(terminals.live_count(), 1);
    terminals.detach(working.id, token);
    terminals.close(working.id, "done").await;
}

#[tokio::test]
async fn closing_a_terminal_kills_it_announces_the_end_and_releases_the_task() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let terminals = world.terminals(8);
    let mut ended = terminals.subscribe_ended();

    let view = terminals.open(task.id, None, false, None, None, TerminalMode::Work).await.unwrap();
    let recorder = Arc::new(Recorder::default());
    terminals.attach(view.id, recorder.clone()).unwrap();
    assert!(terminals.has_live_terminal_for(task.id));

    let closed = terminals.close(view.id, "Closed from the console.").await.unwrap();
    assert_eq!(closed.id, view.id);
    assert_eq!(ended.recv().await.unwrap().terminal_id, view.id);
    let (_, detail) = recorder.ended.lock().unwrap().clone().unwrap();
    assert_eq!(detail, "Closed from the console.");
    assert!(!terminals.has_live_terminal_for(task.id));
    assert!(terminals.close(view.id, "again").await.is_none(), "closing is idempotent");
}

#[tokio::test]
async fn a_late_pane_is_repainted_from_the_scrollback() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let terminals = world.terminals(8);
    let view = terminals.open(task.id, None, false, None, None, TerminalMode::Work).await.unwrap();

    let first = Arc::new(Recorder::default());
    terminals.attach(view.id, first.clone()).unwrap();
    terminals.write(view.id, b"before\n").unwrap();
    eventually("the first echo", || async { first.text().contains("GOT:before") }).await;

    let late = Arc::new(Recorder::default());
    terminals.attach(view.id, late.clone()).unwrap();
    assert!(late.text().contains("ARGS:/rk project:alpha task:one"));
    assert!(late.text().contains("GOT:before"));
    terminals.shutdown().await;
    assert_eq!(late.ended.lock().unwrap().clone().unwrap().1, "The app was shutting down.");
    assert_eq!(terminals.live_count(), 0);
}

#[tokio::test]
async fn opens_past_the_cap_a_missing_folder_or_a_missing_cli_are_refused() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let one = world.task(&project, "one").await;
    let two = world.task(&project, "two").await;
    let terminals = world.terminals(1);
    terminals.open(one.id, None, false, None, None, TerminalMode::Work).await.unwrap();
    let refused = terminals.open(two.id, None, false, None, None, TerminalMode::Work).await.unwrap_err();
    assert_eq!(refused.message(), "Rekall is already running 1 terminals. Close one before opening another.");
    terminals.shutdown().await;

    let gone = world.project("beta", Some(&world.folder.path().join("missing"))).await;
    let task = world.task(&gone, "one").await;
    let refused = world.terminals(8).open(task.id, None, false, None, None, TerminalMode::Work).await.unwrap_err();
    assert!(matches!(refused, RekallError::Conflict(_)));
    assert!(refused.message().ends_with("is not there. Set it again on the project page."), "{}", refused.message());

    std::fs::remove_file(&world.claude).unwrap();
    let refused = world.terminals(8).open(two.id, None, false, None, None, TerminalMode::Work).await.unwrap_err();
    assert_eq!(
        refused.message(),
        "Claude Code's command line tool isn't on this machine. Install it, then try again."
    );
}
