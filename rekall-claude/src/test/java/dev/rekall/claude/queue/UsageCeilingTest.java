package dev.rekall.claude.queue;

import dev.rekall.claude.ClaudeUsageView;
import dev.rekall.claude.ClaudeUsageView.Limit;
import dev.rekall.claude.ClaudeUsageView.Severity;
import dev.rekall.claude.queue.UsageCeiling.Verdict;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.time.Instant;
import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

/** Which usage windows hold the run queue, and until when. */
class UsageCeilingTest {

    private static final Instant NOW = Instant.parse("2026-09-23T10:00:00Z");
    private static final Instant SESSION_RESET = NOW.plusSeconds(2 * 3600);
    private static final Instant WEEK_RESET = NOW.plusSeconds(3 * 24 * 3600);

    private static Limit limit(String key, double percent, Instant resetsAt) {
        return new Limit(key, key, percent, Severity.NORMAL, resetsAt);
    }

    private static ClaudeUsageView reading(Limit... limits) {
        return ClaudeUsageView.ok(List.of(limits), NOW);
    }

    @Test
    @DisplayName("no ceiling never holds, whatever the reading")
    void noCeiling() {
        Verdict verdict = UsageCeiling.check(ClaudeUsageView.unauthenticated(), null, null, NOW);

        assertThat(verdict.clear()).isTrue();
    }

    @Test
    @DisplayName("under the ceiling on every window that counts goes on")
    void underCeiling() {
        Verdict verdict = UsageCeiling.check(
                reading(limit("session", 40, SESSION_RESET), limit("weekly_all", 70, WEEK_RESET)), 80, null, NOW);

        assertThat(verdict.clear()).isTrue();
    }

    @Test
    @DisplayName("the session window at the ceiling holds until it resets, plus the grace")
    void sessionAtCeiling() {
        Verdict verdict = UsageCeiling.check(
                reading(limit("session", 80, SESSION_RESET), limit("weekly_all", 20, WEEK_RESET)), 80, null, NOW);

        assertThat(verdict.clear()).isFalse();
        assertThat(verdict.resumeAt()).isEqualTo(SESSION_RESET.plus(UsageCeiling.RESET_GRACE));
        assertThat(verdict.reason()).contains("session").contains("80%");
    }

    @Test
    @DisplayName("with two windows over, the queue waits for the later reset")
    void latestResetWins() {
        Verdict verdict = UsageCeiling.check(
                reading(limit("session", 95, SESSION_RESET), limit("weekly_all", 90, WEEK_RESET)), 85, null, NOW);

        assertThat(verdict.resumeAt()).isEqualTo(WEEK_RESET.plus(UsageCeiling.RESET_GRACE));
    }

    @Test
    @DisplayName("a model's weekly window counts only when the queue runs that model")
    void modelWindow() {
        ClaudeUsageView opusSpent = reading(limit("session", 10, SESSION_RESET), limit("weekly_opus", 99, WEEK_RESET));

        assertThat(UsageCeiling.check(opusSpent, 90, null, NOW).clear()).isTrue();
        assertThat(UsageCeiling.check(opusSpent, 90, "sonnet", NOW).clear()).isTrue();
        assertThat(UsageCeiling.check(opusSpent, 90, "opus", NOW).clear()).isFalse();
    }

    @Test
    @DisplayName("an unreadable reading holds a queue that has a ceiling, and looks again soon")
    void unreadable() {
        Verdict verdict = UsageCeiling.check(ClaudeUsageView.unavailable(), 80, null, NOW);

        assertThat(verdict.clear()).isFalse();
        assertThat(verdict.resumeAt()).isEqualTo(NOW.plus(UsageCeiling.UNREADABLE_RECHECK));
    }

    @Test
    @DisplayName("a window over the ceiling with no reset time is looked at again later")
    void unknownReset() {
        Verdict verdict = UsageCeiling.check(reading(limit("session", 90, null)), 80, null, NOW);

        assertThat(verdict.resumeAt()).isEqualTo(NOW.plus(UsageCeiling.NO_RESET_RECHECK));
    }

    @Test
    @DisplayName("a reset already in the past still waits out the grace rather than resuming at once")
    void resetInThePast() {
        Verdict verdict = UsageCeiling.check(reading(limit("session", 90, NOW.minusSeconds(600))), 80, null, NOW);

        assertThat(verdict.resumeAt()).isEqualTo(NOW.plus(UsageCeiling.RESET_GRACE));
    }
}
