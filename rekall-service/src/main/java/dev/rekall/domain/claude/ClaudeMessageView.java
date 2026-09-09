package dev.rekall.domain.claude;

import java.time.Instant;
import java.util.UUID;

public record ClaudeMessageView(
        UUID id,
        UUID sessionId,
        int seq,
        ClaudeMessageRole role,
        String content,
        String toolName,
        String meta,
        Instant createdAt) {

    public static ClaudeMessageView of(ClaudeMessage message) {
        return new ClaudeMessageView(
                message.getId(),
                message.getSession().getId(),
                message.getSeq(),
                message.getRole(),
                message.getContent(),
                message.getToolName(),
                message.getMeta(),
                message.getCreatedAt());
    }
}
