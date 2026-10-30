use rekall_kernel::Id;

use super::EndedBus;

#[tokio::test]
async fn every_subscriber_hears_an_announced_end() {
    let bus = EndedBus::new();
    let mut first = bus.subscribe();
    let mut second = bus.subscribe();
    let (session, task) = (Id::random(), Id::random());

    bus.announce(session, Some(task));

    for heard in [first.recv().await.unwrap(), second.recv().await.unwrap()] {
        assert_eq!(heard.terminal_id, session);
        assert_eq!(heard.task_id, Some(task));
    }
}

#[test]
fn an_end_nobody_listens_to_is_not_an_error() {
    EndedBus::new().announce(Id::random(), None);
}
