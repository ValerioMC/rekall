package dev.rekall.claude;

import java.time.Instant;
import java.util.List;

/**
 * What the console needs to draw the Claude usage meter: one row per limit, plus a {@code status}
 * saying whether the numbers are real ({@code OK}), unusable for lack of a token
 * ({@code UNAUTHENTICATED}), or stale because Anthropic was unreachable ({@code UNAVAILABLE}).
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

    /** One consumption window. {@code percent} is 0-100, already clamped; {@code resetsAt} may be null. */
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
