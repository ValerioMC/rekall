use super::*;

#[test]
fn value_of_reads_the_stored_name() {
    assert_eq!(RunQueueItemState::value_of("SKIPPED"), Some(RunQueueItemState::Skipped));
}
