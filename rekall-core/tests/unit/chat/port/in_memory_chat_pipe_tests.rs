use std::sync::Arc;

use rekall_kernel::Instant;

use super::*;
use crate::chat::RecordingChatListener;
use crate::claude::{SessionKind, TerminalMode};

fn view() -> TerminalView {
    let now = Instant::now();
    TerminalView {
        id: Id::random(),
        project_id: Id::random(),
        task_id: None,
        step_id: None,
        anchors: "project:vega".into(),
        working_dir: "/work".into(),
        project_label: "vega".into(),
        task_code: None,
        task_title: None,
        mode: TerminalMode::Work,
        skip_permissions: false,
        model: None,
        effort: None,
        kind: SessionKind::Chat,
        live: true,
        awaiting_permission: false,
        started_at: now,
        last_activity_at: now,
        connections: Vec::new(),
    }
}

fn text(word: &str) -> ChatBody {
    ChatBody::UserMessage { text: word.into() }
}

#[test]
fn attaching_after_a_sequence_replays_only_the_newer_events() {
    let pipe = InMemoryChatPipe::new();
    let session = view();
    pipe.add_session(session.clone());
    for word in ["one", "two", "three"] {
        pipe.emit(session.id, text(word)).unwrap();
    }

    let attached = pipe.attach(session.id, Arc::new(RecordingChatListener::new()), 2).unwrap();

    assert_eq!(attached.backlog.iter().map(|event| event.seq).collect::<Vec<_>>(), vec![3]);
}

#[test]
fn an_attached_listener_hears_new_events_until_it_detaches() {
    let pipe = InMemoryChatPipe::new();
    let session = view();
    pipe.add_session(session.clone());
    let listener = Arc::new(RecordingChatListener::new());
    let attached = pipe.attach(session.id, listener.clone(), 0).unwrap();

    pipe.emit(session.id, text("heard")).unwrap();
    pipe.detach(session.id, attached.token);
    pipe.emit(session.id, text("missed")).unwrap();

    assert_eq!(listener.events().len(), 1);
}

#[test]
fn what_a_client_does_is_recorded_and_an_unknown_session_is_a_conflict() {
    let pipe = InMemoryChatPipe::new();
    let session = view();
    pipe.add_session(session.clone());

    pipe.send(session.id, "hello").unwrap();
    pipe.interrupt(session.id).unwrap();
    pipe.answer_permission(session.id, "req-1", true).unwrap();

    assert_eq!(
        pipe.calls(),
        vec![
            ChatCall::Sent { id: session.id, text: "hello".into() },
            ChatCall::Interrupted { id: session.id },
            ChatCall::Answered { id: session.id, request_id: "req-1".into(), allow: true },
        ]
    );
    assert!(matches!(pipe.send(Id::random(), "x"), Err(RekallError::Conflict(_))));
}
