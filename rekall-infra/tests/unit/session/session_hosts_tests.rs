use std::sync::Arc;
use std::time::Duration;

use rekall_core::claude::{InMemoryTerminalHost, SessionKind, TerminalMode, TerminalTarget};

use super::*;

struct Hosts {
    hosts: SessionHosts,
    terminals: Arc<InMemoryTerminalHost>,
    chats: Arc<InMemoryTerminalHost>,
}

fn hosts(max_sessions: usize) -> Hosts {
    let terminals = Arc::new(InMemoryTerminalHost::new());
    let chats = Arc::new(InMemoryTerminalHost::new());
    let hosts = SessionHosts::start(terminals.clone(), chats.clone(), max_sessions);
    Hosts { hosts, terminals, chats }
}

fn opening(target: TerminalTarget, kind: SessionKind) -> TerminalOpening {
    TerminalOpening { target, step_id: None, skip_permissions: false, model: None, effort: None, mode: TerminalMode::Work, kind }
}

#[tokio::test]
async fn an_opening_goes_to_the_host_of_its_kind() {
    let world = hosts(5);

    let chat = world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Chat)).await.unwrap();
    let terminal = world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Terminal)).await.unwrap();

    assert_eq!(world.chats.list(), vec![chat]);
    assert_eq!(world.terminals.list(), vec![terminal]);
}

#[tokio::test]
async fn the_list_merges_both_kinds_oldest_first() {
    let world = hosts(5);
    let first = world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Chat)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(2)).await;
    let second = world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Terminal)).await.unwrap();

    let listed: Vec<Id> = world.hosts.list().iter().map(|view| view.id).collect();

    assert_eq!(listed, vec![first.id, second.id]);
}

#[tokio::test]
async fn the_cap_counts_both_kinds_together() {
    let world = hosts(2);
    world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Chat)).await.unwrap();
    world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Terminal)).await.unwrap();

    let refused = world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Chat)).await.unwrap_err();

    assert!(matches!(refused, RekallError::Conflict(_)));
    assert_eq!(refused.message(), "Rekall is already running 2 terminals. Close one before opening another.");
    assert_eq!(world.chats.list().len(), 1);
}

#[tokio::test]
async fn an_opening_that_reuses_a_session_at_the_cap_is_not_refused() {
    let world = hosts(1);
    let task = TerminalTarget::Task(Id::random());
    world.hosts.open(opening(task, SessionKind::Chat)).await.unwrap();

    world.hosts.open(opening(task, SessionKind::Chat)).await.unwrap();

    assert_eq!(world.chats.openings().len(), 2);
}

#[tokio::test]
async fn a_session_is_found_closed_and_renewed_on_the_host_that_holds_it() {
    let world = hosts(5);
    let chat = world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Chat)).await.unwrap();
    let terminal = world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Terminal)).await.unwrap();

    assert_eq!(world.hosts.get(chat.id), Some(chat.clone()));
    let renewed_chat = world.hosts.renew(chat.id).await.unwrap();
    world.hosts.renew(terminal.id).await.unwrap();
    assert!(world.hosts.close(renewed_chat.id, "done").await.is_some());

    assert_eq!(world.chats.renewed(), vec![chat.id]);
    assert_eq!(world.terminals.renewed(), vec![terminal.id]);
    assert_eq!(renewed_chat.kind, SessionKind::Chat);
    assert!(world.chats.list().is_empty());
    assert_eq!(world.terminals.list().len(), 1);
}

#[tokio::test]
async fn closing_or_renewing_a_session_nobody_holds_answers_so() {
    let world = hosts(5);

    assert!(world.hosts.close(Id::random(), "gone").await.is_none());
    assert!(matches!(world.hosts.renew(Id::random()).await.unwrap_err(), RekallError::Conflict(_)));
}

#[tokio::test]
async fn a_task_is_live_when_either_host_has_a_session_on_it() {
    let world = hosts(5);
    let on_chat = Id::random();
    let on_terminal = Id::random();
    world.hosts.open(opening(TerminalTarget::Task(on_chat), SessionKind::Chat)).await.unwrap();
    world.hosts.open(opening(TerminalTarget::Task(on_terminal), SessionKind::Terminal)).await.unwrap();

    assert!(world.hosts.has_live_for(on_chat));
    assert!(world.hosts.has_live_for(on_terminal));
    assert!(!world.hosts.has_live_for(Id::random()));
    assert_eq!(world.hosts.work_session_for(on_chat).map(|view| view.kind), Some(SessionKind::Chat));
    assert_eq!(world.hosts.work_session_for(on_terminal).map(|view| view.kind), Some(SessionKind::Terminal));
}

#[tokio::test]
async fn an_end_on_either_host_reaches_one_subscriber() {
    let world = hosts(5);
    let mut ends = world.hosts.subscribe_ended();
    let chat = world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Chat)).await.unwrap();
    let terminal = world.hosts.open(opening(TerminalTarget::Task(Id::random()), SessionKind::Terminal)).await.unwrap();

    world.chats.end(chat.id);
    world.terminals.end(terminal.id);

    let first = tokio::time::timeout(Duration::from_secs(2), ends.recv()).await.unwrap().unwrap();
    let second = tokio::time::timeout(Duration::from_secs(2), ends.recv()).await.unwrap().unwrap();
    let mut heard = [first.terminal_id, second.terminal_id];
    let mut wanted = [chat.id, terminal.id];
    heard.sort_by_key(|id| id.to_string());
    wanted.sort_by_key(|id| id.to_string());
    assert_eq!(heard, wanted);
}
