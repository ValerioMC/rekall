use rekall_core::chat::RecordingChatListener;

use super::*;

fn text(words: &str) -> ChatBody {
    ChatBody::UserMessage { text: words.to_string() }
}

fn seqs(events: &[ChatEvent]) -> Vec<u64> {
    events.iter().map(|event| event.seq).collect()
}

#[test]
fn events_are_numbered_from_one_in_the_order_they_are_published() {
    let mut journal = EventJournal::new(10_000);
    assert_eq!(journal.publish(text("a")).seq, 1);
    assert_eq!(journal.publish(text("b")).seq, 2);
}

#[test]
fn attaching_after_a_seq_returns_only_the_newer_events() {
    let mut journal = EventJournal::new(10_000);
    for words in ["a", "b", "c"] {
        journal.publish(text(words));
    }

    let backlog = journal.attach(1, Arc::new(RecordingChatListener::default()), 1);

    assert_eq!(seqs(&backlog), vec![2, 3]);
}

#[test]
fn a_listener_follows_what_is_published_after_it_attached_and_not_after_it_detached() {
    let mut journal = EventJournal::new(10_000);
    let listener = Arc::new(RecordingChatListener::default());
    journal.attach(7, listener.clone(), 0);

    journal.publish(text("seen"));
    journal.detach(7);
    journal.publish(text("unseen"));

    assert_eq!(seqs(&listener.events()), vec![1]);
}

#[test]
fn the_ring_drops_the_oldest_events_past_its_budget_and_keeps_the_numbering() {
    let mut journal = EventJournal::new(300);
    for _ in 0..20 {
        journal.publish(text("0123456789012345678901234567890123456789"));
    }

    let backlog = journal.attach(1, Arc::new(RecordingChatListener::default()), 0);

    assert!(backlog.len() < 20 && !backlog.is_empty());
    assert_eq!(backlog.last().map(|event| event.seq), Some(20));
    assert_eq!(seqs(&backlog), (21 - backlog.len() as u64..=20).collect::<Vec<_>>());
}

#[test]
fn an_event_larger_than_the_whole_budget_is_still_kept() {
    let mut journal = EventJournal::new(10);

    journal.publish(text("far more than ten bytes of text"));

    assert_eq!(journal.attach(1, Arc::new(RecordingChatListener::default()), 0).len(), 1);
}

#[test]
fn taking_the_listeners_leaves_none_to_follow() {
    let mut journal = EventJournal::new(10_000);
    let listener = Arc::new(RecordingChatListener::default());
    journal.attach(1, listener.clone(), 0);

    assert_eq!(journal.take_listeners().len(), 1);
    journal.publish(text("late"));

    assert!(listener.events().is_empty());
}
