package dev.rekall.claude;

import jakarta.validation.constraints.NotBlank;

import java.util.UUID;

public final class ClaudeApiDtos {

    private ClaudeApiDtos() {
    }

    /**
     * {@code stepId} is optional; when set, the session opens on that step and the process
     * manager moves it to {@code RUNNING} for as long as the session is live.
     * {@code model} is one of {@code sonnet} / {@code fable} / {@code opus} / {@code haiku}, and
     * {@code effort} one of {@code low} / {@code medium} / {@code high} / {@code xhigh} /
     * {@code max}; either null/blank leaves that account default to stand.
     */
    public record StartSessionRequest(UUID stepId, boolean skipPermissions, String model, String effort) {
    }

    public record PromptRequest(@NotBlank String text) {
    }
}
