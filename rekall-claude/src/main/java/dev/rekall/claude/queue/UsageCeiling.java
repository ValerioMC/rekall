package dev.rekall.claude.queue;

import dev.rekall.claude.ClaudeUsageView;
import dev.rekall.claude.ClaudeUsageView.Limit;

import java.time.Duration;
import java.time.Instant;
import java.util.Comparator;
import java.util.List;
import java.util.Objects;
import java.util.Set;

/**
 * Decides whether the run queue may start something new, from a usage reading and the ceiling.
 *
 * <p>The windows that count are the ones a session on the queue's model spends: the five-hour
 * session and the weekly total always, and the weekly Opus or Sonnet window when the queue names
 * that model. A window at or over the ceiling holds the queue until the latest reset among the
 * windows over it, plus {@link #RESET_GRACE} so the next reading is taken after Anthropic has
 * rolled it over. A window over the ceiling with no reset time is looked at again after
 * {@link #NO_RESET_RECHECK}.
 *
 * <p>With a ceiling set and no usable reading (signed out, never reached), the queue holds and
 * looks again after {@link #UNREADABLE_RECHECK}: a ceiling nobody can check is not one to run past.
 */
final class UsageCeiling {

    static final Duration RESET_GRACE = Duration.ofMinutes(1);
    static final Duration NO_RESET_RECHECK = Duration.ofMinutes(15);
    static final Duration UNREADABLE_RECHECK = Duration.ofMinutes(5);

    private static final Set<String> ALWAYS = Set.of("session", "weekly_all");

    private UsageCeiling() {
    }

    /** {@code clear} lets the queue go on; otherwise it holds until {@code resumeAt}, for {@code reason}. */
    record Verdict(boolean clear, Instant resumeAt, String reason) {

        static Verdict go() {
            return new Verdict(true, null, null);
        }

        static Verdict hold(Instant resumeAt, String reason) {
            return new Verdict(false, resumeAt, reason);
        }
    }

    static Verdict check(ClaudeUsageView usage, Integer ceilingPercent, String model, Instant now) {
        if (ceilingPercent == null) {
            return Verdict.go();
        }
        if (usage == null || usage.status() != ClaudeUsageView.Status.OK || usage.limits().isEmpty()) {
            return Verdict.hold(now.plus(UNREADABLE_RECHECK),
                    "Claude usage can't be read, so the %d%% ceiling can't be checked.".formatted(ceilingPercent));
        }
        List<Limit> over = usage.limits().stream()
                .filter(limit -> counts(limit.key(), model))
                .filter(limit -> limit.percent() >= ceilingPercent)
                .toList();
        if (over.isEmpty()) {
            return Verdict.go();
        }
        Limit worst = over.stream().max(Comparator.comparingDouble(Limit::percent)).orElseThrow();
        String reason = "%s is at %d%%, at or over the %d%% ceiling.".formatted(
                worst.label(), Math.round(worst.percent()), ceilingPercent);
        boolean everyResetKnown = over.stream().map(Limit::resetsAt).allMatch(Objects::nonNull);
        if (!everyResetKnown) {
            return Verdict.hold(now.plus(NO_RESET_RECHECK), reason);
        }
        Instant latestReset = over.stream().map(Limit::resetsAt).max(Comparator.naturalOrder()).orElseThrow();
        Instant resumeAt = latestReset.plus(RESET_GRACE);
        return Verdict.hold(resumeAt.isAfter(now) ? resumeAt : now.plus(RESET_GRACE), reason);
    }

    private static boolean counts(String windowKey, String model) {
        if (ALWAYS.contains(windowKey)) {
            return true;
        }
        return model != null && windowKey.equals("weekly_" + model);
    }
}
