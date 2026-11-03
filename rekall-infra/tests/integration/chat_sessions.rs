//! The chat session manager against a stand-in headless `claude` that replays a fixture: opening,
//! reuse, renewal, the running marker, the idle sweep, shutdown, a crash, and a client that
//! reattaches from the last event it saw.

mod support;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rekall_core::chat::{ChatBody, ChatEvent, ChatPipe, RecordingChatListener};
use rekall_core::claude::{SessionKind, TerminalHost, TerminalMode, TerminalOpening, TerminalTarget};
use rekall_infra::chat::ChatSessionManager;
use rekall_infra::cli::{ClaudeCli, EnvironmentSource};
use rekall_infra::ClaudeConfig;
use rekall_kernel::{Id, RekallError};
use rekall_store::{task, task_step, TaskStepState};
use support::{eventually, world, World};

const FAKE_CLAUDE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fake_claude.sh");
const PLAIN_REPLY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/chat/plain_reply.jsonl");
const WAITING_FOR_PERMISSION: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/chat/permission_waiting.jsonl");
const PERMISSION_THEN_END_OF_TURN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/chat/permission_request_allow.jsonl");

struct FakeClaudeEnvironment(HashMap<String, String>);

#[async_trait::async_trait]
impl EnvironmentSource for FakeClaudeEnvironment {
    async fn current(&self) -> HashMap<String, String> {
        self.0.clone()
    }
}

struct Chats {
    manager: Arc<ChatSessionManager>,
    received: PathBuf,
}

impl Chats {
    fn received_lines(&self) -> Vec<String> {
        std::fs::read_to_string(&self.received).unwrap_or_default().lines().map(str::to_string).collect()
    }
}

fn chats(world: &World, max_sessions: usize, idle_minutes: u64, behaviour: &[(&str, &str)]) -> Chats {
    let received = world.home.path().join("received.txt");
    let mut environment = HashMap::from([
        ("PATH".to_string(), "/usr/bin:/bin".to_string()),
        ("FAKE_CLAUDE_FIXTURE".to_string(), PLAIN_REPLY.to_string()),
        ("FAKE_CLAUDE_RECEIVED".to_string(), received.to_string_lossy().into_owned()),
    ]);
    environment.extend(behaviour.iter().map(|(key, value)| (key.to_string(), value.to_string())));
    let config = ClaudeConfig { max_sessions, idle_minutes, home: world.home.path().to_path_buf(), ..ClaudeConfig::default() };
    let cli = ClaudeCli::new(Some(FAKE_CLAUDE), world.home.path().to_path_buf(), Arc::new(FakeClaudeEnvironment(environment)));
    let manager = ChatSessionManager::new(
        world.services.terminal_launch.clone(),
        cli,
        world.work_marker(),
        world.broker("http://127.0.0.1:0"),
        world.telemetry(),
        &config,
    );
    Chats { manager, received }
}

/// A client attached from the start: the backlog it was handed plus what it was told after.
struct Follower {
    backlog: Vec<ChatEvent>,
    listener: Arc<RecordingChatListener>,
}

impl Follower {
    fn events(&self) -> Vec<ChatEvent> {
        self.backlog.iter().cloned().chain(self.listener.events()).collect()
    }

    fn turn_finished(&self) -> bool {
        self.events().iter().any(|event| matches!(event.body, ChatBody::TurnFinished { .. }))
    }

    fn requested_permission(&self) -> Option<String> {
        self.events().into_iter().find_map(|event| match event.body {
            ChatBody::PermissionRequested { request_id, .. } => Some(request_id),
            _ => None,
        })
    }

    fn resolutions(&self) -> Vec<(String, bool)> {
        self.events()
            .into_iter()
            .filter_map(|event| match event.body {
                ChatBody::PermissionResolved { request_id, allowed } => Some((request_id, allowed)),
                _ => None,
            })
            .collect()
    }
}

fn follow(chats: &Chats, id: Id) -> Follower {
    let listener = Arc::new(RecordingChatListener::new());
    let backlog = chats.manager.attach(id, listener.clone(), 0).unwrap().backlog;
    Follower { backlog, listener }
}

fn work_on(task: &task::Model, step: Option<&task_step::Model>) -> TerminalOpening {
    TerminalOpening {
        target: TerminalTarget::Task(task.id),
        step_id: step.map(|step| step.id),
        skip_permissions: true,
        model: Some("opus".into()),
        effort: None,
        mode: TerminalMode::Work,
        kind: SessionKind::Chat,
    }
}

#[tokio::test]
async fn a_chat_session_starts_claude_in_the_project_folder_marks_the_step_and_streams_the_first_turn() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Open).await;
    let chats = chats(&world, 8, 120, &[]);

    let view = chats.manager.open(work_on(&task, Some(&step))).await.unwrap();

    assert_eq!((view.kind, view.live, view.step_id), (SessionKind::Chat, true, Some(step.id)));
    assert_eq!((view.anchors.as_str(), view.model.as_deref()), ("project:alpha task:one", Some("opus")));
    assert_eq!(world.step_state(&task, &step).await, TaskStepState::Running);
    let follower = follow(&chats, view.id);
    eventually("the first turn to finish", || async { follower.turn_finished() }).await;
    let events = follower.events();
    assert_eq!(events[0].body, ChatBody::UserMessage { text: "/rk project:alpha task:one".into() });
    assert_eq!(events.iter().map(|event| event.seq).collect::<Vec<_>>(), (1..=events.len() as u64).collect::<Vec<_>>());
    assert!(chats.received_lines()[0].contains("/rk project:alpha task:one"));
}

#[tokio::test]
async fn a_second_open_on_the_task_reuses_the_session_and_moves_the_running_marker() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let first = world.step(&task, "First", 0, TaskStepState::Open).await;
    let second = world.step(&task, "Second", 1, TaskStepState::Open).await;
    let chats = chats(&world, 8, 120, &[]);
    let view = chats.manager.open(work_on(&task, Some(&first))).await.unwrap();

    let again = chats.manager.open(work_on(&task, Some(&second))).await.unwrap();

    assert_eq!(again.id, view.id);
    assert_eq!(again.step_id, Some(second.id));
    assert_eq!(chats.manager.live_count(), 1);
    assert_eq!(world.step_state(&task, &first).await, TaskStepState::Open);
    assert_eq!(world.step_state(&task, &second).await, TaskStepState::Running);
    eventually("the work line sent again", || async { chats.received_lines().len() == 2 }).await;
}

#[tokio::test]
async fn renewing_a_work_session_closes_it_and_opens_another_with_the_same_choices() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Open).await;
    let chats = chats(&world, 8, 120, &[]);
    let view = chats.manager.open(work_on(&task, Some(&step))).await.unwrap();
    let mut ended = chats.manager.subscribe_ended();

    let renewed = chats.manager.renew(view.id).await.unwrap();

    assert_ne!(renewed.id, view.id);
    assert_eq!((renewed.kind, renewed.model.as_deref(), renewed.skip_permissions), (SessionKind::Chat, Some("opus"), true));
    assert_eq!(ended.recv().await.unwrap().terminal_id, view.id);
    assert!(chats.manager.get(view.id).is_none());
    assert_eq!(chats.manager.work_session_for(task.id).map(|work| work.id), Some(renewed.id));
}

#[tokio::test]
async fn only_a_session_working_steps_can_be_renewed() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let chats = chats(&world, 8, 120, &[]);
    let plan = TerminalOpening { mode: TerminalMode::Plan, ..work_on(&task, None) };
    let view = chats.manager.open(plan).await.unwrap();

    let refused = chats.manager.renew(view.id).await.unwrap_err();

    assert!(matches!(refused, RekallError::IllegalArgument(_)), "{refused:?}");
    assert!(chats.manager.work_session_for(task.id).is_none());
    assert!(chats.manager.has_live_for(task.id));
}

#[tokio::test]
async fn the_idle_sweep_closes_a_session_with_no_activity_and_releases_its_step() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Open).await;
    let chats = chats(&world, 8, 0, &[]);
    let view = chats.manager.open(work_on(&task, Some(&step))).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    chats.manager.sweep_idle().await;

    assert!(chats.manager.get(view.id).is_none());
    assert_eq!(world.step_state(&task, &step).await, TaskStepState::Open);
}

#[tokio::test]
async fn the_idle_sweep_leaves_a_session_waiting_on_a_permission_prompt_open() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let chats = chats(&world, 8, 0, &[("FAKE_CLAUDE_FIXTURE", WAITING_FOR_PERMISSION)]);
    let view = chats.manager.open(work_on(&task, None)).await.unwrap();
    let follower = follow(&chats, view.id);
    eventually("the permission request", || async { follower.requested_permission().is_some() }).await;
    assert!(chats.manager.get(view.id).unwrap().awaiting_permission);
    tokio::time::sleep(Duration::from_millis(50)).await;

    chats.manager.sweep_idle().await;

    assert!(chats.manager.get(view.id).is_some());
}

#[tokio::test]
async fn shutdown_ends_every_session_tells_its_clients_and_releases_the_markers() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let one = world.task(&project, "one").await;
    let two = world.task(&project, "two").await;
    let step_one = world.step(&one, "First", 0, TaskStepState::Open).await;
    let step_two = world.step(&two, "First", 0, TaskStepState::Open).await;
    let chats = chats(&world, 8, 120, &[]);
    let first = chats.manager.open(work_on(&one, Some(&step_one))).await.unwrap();
    chats.manager.open(work_on(&two, Some(&step_two))).await.unwrap();
    let listener = Arc::new(RecordingChatListener::new());
    chats.manager.attach(first.id, listener.clone(), 0).unwrap();

    chats.manager.shutdown().await;

    assert_eq!(chats.manager.live_count(), 0);
    assert_eq!(world.step_state(&one, &step_one).await, TaskStepState::Open);
    assert_eq!(world.step_state(&two, &step_two).await, TaskStepState::Open);
    assert_eq!(listener.end().map(|(_, detail)| detail), Some("The app was shutting down.".to_string()));
    assert!(matches!(chats.manager.send(first.id, "late"), Err(RekallError::Conflict(_))));
}

#[tokio::test]
async fn a_process_that_crashes_releases_its_step_and_announces_the_end() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Open).await;
    let chats = chats(&world, 8, 120, &[("FAKE_CLAUDE_EXIT_CODE", "3"), ("FAKE_CLAUDE_EXIT_DELAY", "0.5")]);
    let mut ended = chats.manager.subscribe_ended();
    let view = chats.manager.open(work_on(&task, Some(&step))).await.unwrap();
    let follower = follow(&chats, view.id);

    let end = tokio::time::timeout(Duration::from_secs(10), ended.recv()).await.unwrap().unwrap();

    assert_eq!((end.terminal_id, end.task_id), (view.id, Some(task.id)));
    assert_eq!(follower.listener.end(), Some((3, "claude exited with code 3".to_string())));
    assert!(matches!(follower.events().last().map(|event| &event.body), Some(ChatBody::Ended { exit_code: 3, .. })));
    assert_eq!(chats.manager.live_count(), 0);
    assert_eq!(world.step_state(&task, &step).await, TaskStepState::Open);
}

#[tokio::test]
async fn attaching_after_a_seq_returns_only_the_newer_events() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let chats = chats(&world, 8, 120, &[]);
    let view = chats.manager.open(work_on(&task, None)).await.unwrap();
    let first = follow(&chats, view.id);
    eventually("the first turn to finish", || async { first.turn_finished() }).await;
    let seen = first.events().last().map(|event| event.seq).unwrap();

    chats.manager.send(view.id, "and again").unwrap();
    let attached = chats.manager.attach(view.id, Arc::new(RecordingChatListener::new()), seen).unwrap();

    assert!(attached.backlog.iter().all(|event| event.seq > seen));
    assert_eq!(attached.backlog[0].body, ChatBody::UserMessage { text: "and again".into() });
    assert_eq!(attached.backlog[0].seq, seen + 1);
}

#[tokio::test]
async fn a_session_past_the_cap_is_refused_and_a_folder_that_is_gone_is_a_conflict() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let lost = world.project("lost", Some(&world.folder.path().join("missing"))).await;
    let one = world.task(&project, "one").await;
    let two = world.task(&project, "two").await;
    let elsewhere = world.task(&lost, "three").await;
    let chats = chats(&world, 1, 120, &[]);
    chats.manager.open(work_on(&one, None)).await.unwrap();

    let capped = chats.manager.open(work_on(&two, None)).await.unwrap_err();
    let missing = chats.manager.open(work_on(&elsewhere, None)).await.unwrap_err();

    assert!(matches!(capped, RekallError::Conflict(ref said) if said.contains("1 chat sessions")), "{capped:?}");
    assert!(matches!(missing, RekallError::Conflict(ref said) if said.contains("is not there")), "{missing:?}");
    assert_eq!(chats.manager.live_count(), 1);
}

async fn open_waiting_on_permission(world: &World, fixture: &str) -> (Chats, Id, Follower, String) {
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let chats = chats(world, 8, 120, &[("FAKE_CLAUDE_FIXTURE", fixture)]);
    let view = chats.manager.open(work_on(&task, None)).await.unwrap();
    let follower = follow(&chats, view.id);
    eventually("the permission request", || async { follower.requested_permission().is_some() }).await;
    let request_id = follower.requested_permission().unwrap();
    (chats, view.id, follower, request_id)
}

fn answers_received(chats: &Chats) -> Vec<serde_json::Value> {
    chats
        .received_lines()
        .iter()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|line| line["type"] == "control_response")
        .collect()
}

#[tokio::test]
async fn allowing_a_waiting_request_reaches_the_process_and_is_shown_as_resolved() {
    let world = world().await;
    let (chats, id, follower, request_id) = open_waiting_on_permission(&world, WAITING_FOR_PERMISSION).await;

    chats.manager.answer_permission(id, &request_id, true).unwrap();

    eventually("the answer to reach claude", || async { answers_received(&chats).len() == 1 }).await;
    let answer = &answers_received(&chats)[0];
    assert_eq!((answer["response"]["request_id"].as_str(), answer["response"]["response"]["behavior"].as_str()), (Some(request_id.as_str()), Some("allow")));
    assert_eq!(follower.resolutions(), vec![(request_id.clone(), true)]);
    let again = chats.manager.answer_permission(id, &request_id, true).unwrap_err();
    assert!(matches!(again, RekallError::NotFound(_)), "{again:?}");
}

#[tokio::test]
async fn denying_a_waiting_request_tells_the_process_to_refuse_the_call() {
    let world = world().await;
    let (chats, id, follower, request_id) = open_waiting_on_permission(&world, WAITING_FOR_PERMISSION).await;

    chats.manager.answer_permission(id, &request_id, false).unwrap();

    eventually("the answer to reach claude", || async { answers_received(&chats).len() == 1 }).await;
    assert_eq!(answers_received(&chats)[0]["response"]["response"]["behavior"], "deny");
    assert_eq!(follower.resolutions(), vec![(request_id, false)]);
}

#[tokio::test]
async fn a_request_nobody_asked_is_not_found() {
    let world = world().await;
    let (chats, id, _follower, _request_id) = open_waiting_on_permission(&world, WAITING_FOR_PERMISSION).await;

    let refused = chats.manager.answer_permission(id, "no-such-request", true).unwrap_err();

    assert!(matches!(refused, RekallError::NotFound(_)), "{refused:?}");
    assert!(answers_received(&chats).is_empty());
}

#[tokio::test]
async fn closing_a_session_denies_the_requests_still_waiting() {
    let world = world().await;
    let (chats, id, follower, request_id) = open_waiting_on_permission(&world, WAITING_FOR_PERMISSION).await;

    chats.manager.close(id, "Closed from the console.").await.unwrap();

    assert_eq!(follower.resolutions(), vec![(request_id, false)]);
    let kinds: Vec<bool> = follower.events().iter().map(|event| matches!(event.body, ChatBody::Ended { .. })).collect();
    assert_eq!(kinds.last(), Some(&true), "the end comes after the denial");
    assert!(matches!(chats.manager.answer_permission(id, "late", true), Err(RekallError::Conflict(_))));
}

#[tokio::test]
async fn a_turn_that_ends_leaves_no_request_waiting() {
    let world = world().await;
    let (chats, id, follower, request_id) = open_waiting_on_permission(&world, PERMISSION_THEN_END_OF_TURN).await;
    eventually("the turn to finish", || async { follower.turn_finished() }).await;

    let late = chats.manager.answer_permission(id, &request_id, true).unwrap_err();

    assert!(matches!(late, RekallError::NotFound(_)), "{late:?}");
    assert_eq!(follower.resolutions(), vec![(request_id, false)]);
}
