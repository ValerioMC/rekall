use rekall_core::chat::ChatBody;

use super::*;

fn user_message(seq: u64, text: &str) -> ChatEvent {
    ChatEvent { seq, body: ChatBody::UserMessage { text: text.into() } }
}

fn listener() -> (SseListener, mpsc::UnboundedReceiver<Queued>, Arc<AtomicUsize>) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let pending = Arc::new(AtomicUsize::new(0));
    (SseListener::new(sender, pending.clone()), receiver, pending)
}

#[test]
fn an_event_is_queued_under_its_seq_with_its_json_and_counted() {
    let (listener, mut receiver, pending) = listener();

    listener.event(&user_message(7, "hello"));

    let Some(Queued::Frame(frame)) = receiver.try_recv().ok() else { panic!("a frame was expected") };
    assert_eq!(frame.seq, 7);
    assert_eq!(frame.json, r#"{"seq":7,"type":"userMessage","text":"hello"}"#);
    assert_eq!(pending.load(Ordering::SeqCst), frame.json.len());
}

#[test]
fn the_end_of_the_session_closes_the_stream_after_what_was_queued() {
    let (listener, mut receiver, _) = listener();

    listener.event(&user_message(1, "last"));
    listener.ended(0, "done");
    listener.event(&user_message(2, "too late"));

    assert!(matches!(receiver.try_recv(), Ok(Queued::Frame(_))));
    assert!(matches!(receiver.try_recv(), Ok(Queued::Close)));
    assert!(receiver.try_recv().is_err());
}

#[test]
fn a_reader_that_falls_too_far_behind_is_dropped() {
    let (listener, mut receiver, _) = listener();
    let text = "x".repeat(SEND_BUFFER_LIMIT_BYTES / 2);

    listener.event(&user_message(1, &text));
    listener.event(&user_message(2, &text));
    listener.event(&user_message(3, "never queued"));

    assert!(matches!(receiver.try_recv(), Ok(Queued::Frame(_))));
    assert!(matches!(receiver.try_recv(), Ok(Queued::Close)));
    assert!(receiver.try_recv().is_err());
}
