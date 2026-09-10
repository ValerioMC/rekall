package dev.rekall.claude;

import java.time.Instant;
import java.util.UUID;

public final class TerminalApiDtos {

    private TerminalApiDtos() {
    }

    /**
     * What the console sends to open a terminal. {@code stepId}, {@code model} and {@code effort}
     * are optional; an unknown or blank {@code model}/{@code effort} leaves the account default.
     */
    public record OpenTerminalRequest(UUID stepId, boolean skipPermissions, String model, String effort) {
    }

    /** A live terminal as the console sees it, drawn from {@link PtyTerminalManager}'s current set. */
    public record TerminalView(
            UUID id,
            UUID taskId,
            UUID stepId,
            String anchors,
            String workingDir,
            String projectLabel,
            String taskLabel,
            String taskTitle,
            boolean skipPermissions,
            String model,
            String effort,
            boolean live,
            Instant startedAt,
            Instant lastActivityAt) {
    }
}
