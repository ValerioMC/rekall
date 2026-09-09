package dev.rekall.domain.claude;

/**
 * Where a {@link ClaudeSession} is on its line.
 *
 * <p>{@code STARTING} the process is up and loading its context; {@code WORKING} a turn is in
 * flight; {@code READY} it is waiting for the next prompt; {@code EXITED} it was closed, by a
 * person or by the idle sweep or by a clean end; {@code FAILED} the process died on its own with
 * a non-zero code. The first three are the only ones a process is still attached to.
 */
public enum ClaudeSessionStatus {

    STARTING,

    WORKING,

    READY,

    EXITED,

    FAILED;

    public boolean live() {
        return this == STARTING || this == WORKING || this == READY;
    }

    public boolean acceptsPrompt() {
        return this == READY;
    }
}
