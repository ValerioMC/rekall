package dev.rekall.claude;

import jakarta.validation.constraints.NotBlank;

import java.util.UUID;

public final class ClaudeApiDtos {

    private ClaudeApiDtos() {
    }

    /** {@code stepId} is optional and only records which step the session was opened against. */
    public record StartSessionRequest(UUID stepId, boolean skipPermissions) {
    }

    public record PromptRequest(@NotBlank String text) {
    }
}
