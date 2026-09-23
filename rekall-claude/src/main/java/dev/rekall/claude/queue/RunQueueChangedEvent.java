package dev.rekall.claude.queue;

/** The run queue as it stands after a write, pushed to the console once the write commits. */
public record RunQueueChangedEvent(RunQueueView queue) {
}
