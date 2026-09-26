use super::*;
use rekall_service::step::StepStreamEvent;
use rekall_service::wrapup::WrapupStreamEvent;
use rekall_service::DomainEvent;

#[tokio::test]
async fn each_open_connection_is_counted_then_dropped_when_the_app_closes() {
    let stream = StepEventStream::new(EventBus::new());
    let _a = stream.open();
    let _b = stream.open();
    let _c = stream.open();
    assert_eq!(stream.client_count(), 3);
    stream.release_on_shutdown();
    assert_eq!(stream.client_count(), 0, "graceful shutdown must find nothing holding a request open");
}

#[tokio::test]
async fn a_change_after_shutdown_reaches_nobody_and_does_not_fail() {
    let bus = EventBus::new();
    let stream = StepEventStream::new(bus.clone());
    let _open = stream.open();
    stream.release_on_shutdown();
    bus.publish(DomainEvent::Steps(StepStreamEvent { task_id: rekall_common::Id::random(), steps: vec![] }));
    bus.publish(DomainEvent::Wrapup(WrapupStreamEvent { task_id: rekall_common::Id::random(), wrapup: None, deleted: true }));
    assert_eq!(stream.client_count(), 0);
}

#[tokio::test]
async fn shutdown_with_no_console_connected_or_twice_is_harmless() {
    let stream = StepEventStream::new(EventBus::new());
    stream.release_on_shutdown();
    let _open = stream.open();
    stream.release_on_shutdown();
    stream.release_on_shutdown();
    assert_eq!(stream.client_count(), 0);
}
