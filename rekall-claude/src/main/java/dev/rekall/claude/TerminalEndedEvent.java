package dev.rekall.claude;

import java.util.UUID;

/** A terminal has gone, whether it exited, was closed, idled out or went down with the app. */
public record TerminalEndedEvent(UUID terminalId, UUID taskId) {
}
