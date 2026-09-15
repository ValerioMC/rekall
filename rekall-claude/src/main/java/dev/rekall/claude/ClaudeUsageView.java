package dev.rekall.claude;

import java.time.Instant;
import java.util.List;

/**
 * What the console needs to draw the Claude usage meter: one row per limit, plus a {@code status}
 * saying whether the numbers are real ({@code OK}), unusable for lack of a valid token
 * ({@code UNAUTHENTICATED}), held back because Anthropic asked this machine to wait
 * ({@code RATE_LIMITED}), or stale because Anthropic was unreachable ({@code UNAVAILABLE}).
 * {@code retryAt} is set whenever a wait is in force, whatever the status: a last good reading
 * served during one carries it too, so the console knows the figures cannot be refreshed yet.
 */
public record ClaudeUsageView(Status status, List<Limit> limits, Instant fetchedAt, Instant retryAt) {

    public enum Status {
        OK,
        UNAUTHENTICATED,
        RATE_LIMITED,
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

    public static ClaudeUsageView ok(List<Limit> limits, Instant fetchedAt) {
        return new ClaudeUsageView(Status.OK, limits, fetchedAt, null);
    }

    public static ClaudeUsageView unauthenticated() {
        return new ClaudeUsageView(Status.UNAUTHENTICATED, List.of(), Instant.now(), null);
    }

    public static ClaudeUsageView rateLimited(Instant retryAt) {
        return new ClaudeUsageView(Status.RATE_LIMITED, List.of(), Instant.now(), retryAt);
    }

    public static ClaudeUsageView unavailable() {
        return new ClaudeUsageView(Status.UNAVAILABLE, List.of(), Instant.now(), null);
    }

    /** The same reading, marked as one that cannot be refreshed before {@code retryAt}. */
    public ClaudeUsageView heldUntil(Instant retryAt) {
        return new ClaudeUsageView(status, limits, fetchedAt, retryAt);
    }
}
