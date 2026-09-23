package dev.rekall.domain;

/**
 * Where the run queue is on its line. {@code IDLE} runs nothing; {@code SCHEDULED} waits for its
 * start time; {@code RUNNING} has a session on the head item, or is about to open one;
 * {@code HOLDING} has reached the usage ceiling and waits for the window to reset.
 */
public enum RunQueueState {

    IDLE,

    SCHEDULED,

    RUNNING,

    HOLDING;

    /** True while the queue has been started and not stopped or run dry. */
    public boolean armed() {
        return this != IDLE;
    }
}
