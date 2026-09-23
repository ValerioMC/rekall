package dev.rekall.domain;

/**
 * One queued task's outcome. {@code QUEUED} is waiting its turn; {@code RUNNING} has a session on
 * it; the other three are where it stopped: its work claimed, nothing on it to do, or a session
 * that could not open or ended before the work was claimed.
 */
public enum RunQueueItemState {

    QUEUED,

    RUNNING,

    FINISHED,

    SKIPPED,

    FAILED;

    /** True once the queue is finished with the item, whatever the outcome. */
    public boolean settled() {
        return this == FINISHED || this == SKIPPED || this == FAILED;
    }
}
