package dev.rekall.claude;

import java.time.Instant;
import java.util.List;

/**
 * What the console needs to draw the Claude usage meter: one row per limit the account is subject
 * to, plus a status that says whether the numbers are real.
 *
 * <p>{@code status} is the only thing a caller has to branch on. {@code OK} means {@code limits}
 * holds live figures. {@code UNAUTHENTICATED} means Claude Code has no usable token here, so the
 * meter asks the user to sign in. {@code UNAVAILABLE} means the account is fine but Anthropic could
 * not be reached; the meter shows its last shape greyed rather than an error.
 */
public record ClaudeUsageView(Status status, List<Limit> limits, Instant fetchedAt) {

    public enum Status {
        OK,
        UNAUTHENTICATED,
        UNAVAILABLE
    }

    public enum Severity {
        NORMAL,
        WARNING,
        CRITICAL
    }

    /**
     * One consumption window. {@code key} is stable and machine-facing ({@code session},
     * {@code weekly_all}, {@code weekly_opus}, {@code weekly_sonnet}); {@code label} is for display.
     * {@code percent} is 0-100, already clamped. {@code resetsAt} is null when the window has no
     * scheduled reset.
     */
    public record Limit(
            String key, String label, double percent, Severity severity, Instant resetsAt) {
    }

    public static ClaudeUsageView unauthenticated() {
        return new ClaudeUsageView(Status.UNAUTHENTICATED, List.of(), Instant.now());
    }

    public static ClaudeUsageView unavailable() {
        return new ClaudeUsageView(Status.UNAVAILABLE, List.of(), Instant.now());
    }
}
