use rekall_core::chat::{ChatBody, ChatCall, InMemoryChatPipe, RecordingChatListener, ToolInput};
use rekall_core::claude::{InMemoryTerminalHost, TerminalTarget};

use super::*;

struct World {
    service: ChatService,
    pipe: Arc<InMemoryChatPipe>,
    chat: Id,
    terminal: Id,
}

fn world() -> World {
    let host = Arc::new(InMemoryTerminalHost::new());
    let pipe = Arc::new(InMemoryChatPipe::new());
    let target = TerminalTarget::Task(Id::random());
    let chat_view = InMemoryTerminalHost::session_on(target, SessionKind::Chat);
    let terminal_view = InMemoryTerminalHost::session_on(target, SessionKind::Terminal);
    let (chat, terminal) = (chat_view.id, terminal_view.id);
    pipe.add_session(chat_view.clone());
    host.add(chat_view);
    host.add(terminal_view);
    World { service: ChatService::new(host, pipe.clone()), pipe, chat, terminal }
}

#[test]
fn a_message_an_interrupt_and_an_answer_reach_the_session() {
    let world = world();

    world.service.send(world.chat, "go on").unwrap();
    world.service.interrupt(world.chat).unwrap();
    world.service.answer_permission(world.chat, "req-1", true).unwrap();

    assert_eq!(
        world.pipe.calls(),
        vec![
            ChatCall::Sent { id: world.chat, text: "go on".into() },
            ChatCall::Interrupted { id: world.chat },
            ChatCall::Answered { id: world.chat, request_id: "req-1".into(), allow: true },
        ]
    );
}

#[test]
fn a_terminal_session_is_refused_by_every_call() {
    let world = world();
    let listener = Arc::new(RecordingChatListener::new());

    let refusals = [
        world.service.send(world.terminal, "hi").unwrap_err(),
        world.service.interrupt(world.terminal).unwrap_err(),
        world.service.answer_permission(world.terminal, "req-1", false).unwrap_err(),
        world.service.attach(world.terminal, listener, 0).unwrap_err(),
    ];

    assert!(refusals.iter().all(|refusal| matches!(refusal, RekallError::IllegalArgument(_))), "{refusals:?}");
    assert!(world.pipe.calls().is_empty());
}

#[test]
fn an_unknown_session_is_not_found() {
    let world = world();

    let refused = world.service.send(Id::random(), "hi").unwrap_err();

    assert!(matches!(refused, RekallError::NotFound(_)), "{refused:?}");
}

#[test]
fn a_session_the_host_still_lists_but_the_pipe_lost_is_a_conflict() {
    let host = Arc::new(InMemoryTerminalHost::new());
    let view = InMemoryTerminalHost::session_on(TerminalTarget::Task(Id::random()), SessionKind::Chat);
    let id = view.id;
    host.add(view);
    let service = ChatService::new(host, Arc::new(InMemoryChatPipe::new()));

    let refused = service.send(id, "hi").unwrap_err();

    assert!(matches!(refused, RekallError::Conflict(_)), "{refused:?}");
}

#[test]
fn attaching_hands_back_only_the_events_after_the_one_asked_for() {
    let world = world();
    world.pipe.emit(world.chat, ChatBody::UserMessage { text: "one".into() }).unwrap();
    world.pipe.emit(world.chat, ChatBody::ToolUse { tool_use_id: "t".into(), name: "Bash".into(), input: ToolInput::Other { name: "Bash".into(), text: "ls".into() } }).unwrap();
    let listener = Arc::new(RecordingChatListener::new());

    let attachment = world.service.attach(world.chat, listener.clone(), 1).unwrap();
    world.pipe.emit(world.chat, ChatBody::UserMessage { text: "three".into() }).unwrap();

    assert_eq!(attachment.backlog.iter().map(|event| event.seq).collect::<Vec<_>>(), vec![2]);
    assert_eq!(listener.events().iter().map(|event| event.seq).collect::<Vec<_>>(), vec![3]);

    world.service.detach(world.chat, attachment.token);
    world.pipe.emit(world.chat, ChatBody::UserMessage { text: "four".into() }).unwrap();
    assert_eq!(listener.events().len(), 1);
}
