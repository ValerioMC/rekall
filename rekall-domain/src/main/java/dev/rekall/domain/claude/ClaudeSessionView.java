package dev.rekall.domain.claude;

import java.time.Instant;
import java.util.UUID;

public record ClaudeSessionView(
        UUID id,
        UUID taskId,
        String taskLabel,
        String taskTitle,
        String projectLabel,
        String anchor,
        UUID stepId,
        String anchors,
        String workingDir,
        String cliSessionId,
        ClaudeSessionStatus status,
        boolean live,
        boolean skipPermissions,
        String detail,
        Integer exitCode,
        Instant lastActivityAt,
        Instant startedAt,
        Instant endedAt,
        Instant updatedAt) {

    public static ClaudeSessionView of(ClaudeSession session) {
        return new ClaudeSessionView(
                session.getId(),
                session.getTask().getId(),
                session.getTask().getLabel(),
                session.getTask().getTitle(),
                session.getTask().getProject().getLabel(),
                "project:%s task:%s".formatted(
                        session.getTask().getProject().getLabel(), session.getTask().getLabel()),
                session.getStepId(),
                session.getAnchors(),
                session.getWorkingDir(),
                session.getCliSessionId(),
                session.getStatus(),
                session.getStatus().live(),
                session.isSkipPermissions(),
                session.getDetail(),
                session.getExitCode(),
                session.getLastActivityAt(),
                session.getStartedAt(),
                session.getEndedAt(),
                session.getUpdatedAt());
    }
}
