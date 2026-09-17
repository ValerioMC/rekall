package dev.rekall.domain.commit;

import java.util.List;
import java.util.Locale;
import java.util.regex.Pattern;

/**
 * Writes the commit message for an automatic commit: a Conventional Commits subject naming what
 * was claimed, and a body that says where it was claimed and which files moved. The type comes
 * from the files alone, since the only thing known about the change is what it touched:
 * {@code docs} when every file is documentation, {@code test} when every file is a test,
 * {@code feat} otherwise.
 */
final class CommitMessageGenerator {

    /** Git shows the subject in full up to here; past it, tools wrap or cut. */
    static final int SUBJECT_MAX = 72;

    /** Paths listed one by one in the body; the rest are counted. */
    static final int LISTED_PATHS_MAX = 20;

    private static final Pattern DOCUMENTATION = Pattern.compile(
            "(?i)(^|/)(docs?|documentation)/|\\.(md|mdx|markdown|rst|txt|adoc)$");

    private static final Pattern TEST = Pattern.compile(
            "(?i)(^|/)(tests?|__tests__|spec|e2e)/|\\.(spec|test)\\.[a-z]+$|Tests?\\.java$|_test\\.(go|py)$|test_[^/]*\\.py$");

    /** What the commit is for: the step or task claimed, as a title, plus where it lives. */
    record Subject(String title, String anchor, Integer stepNumber) {
    }

    private CommitMessageGenerator() {
    }

    static String generate(Subject subject, List<PendingChange> changes) {
        StringBuilder message = new StringBuilder(subjectLine(subject, changes));
        message.append("\n\n").append(subject.anchor());
        if (subject.stepNumber() != null) {
            message.append(", step ").append(subject.stepNumber());
        }
        message.append('\n').append(summary(changes)).append('\n');
        changes.stream().limit(LISTED_PATHS_MAX).forEach(change ->
                message.append("  ").append(marker(change.kind())).append(' ').append(change.path()).append('\n'));
        if (changes.size() > LISTED_PATHS_MAX) {
            message.append("  … and ").append(changes.size() - LISTED_PATHS_MAX).append(" more\n");
        }
        return message.toString();
    }

    static String subjectLine(Subject subject, List<PendingChange> changes) {
        String prefix = typeOf(changes) + ": ";
        String title = subject.title().strip().replaceAll("\\s+", " ");
        int room = SUBJECT_MAX - prefix.length();
        if (title.length() > room) {
            title = title.substring(0, room - 1).stripTrailing() + "…";
        }
        return prefix + title;
    }

    static String typeOf(List<PendingChange> changes) {
        if (!changes.isEmpty() && changes.stream().allMatch(change -> DOCUMENTATION.matcher(change.path()).find())) {
            return "docs";
        }
        if (!changes.isEmpty() && changes.stream().allMatch(change -> TEST.matcher(change.path()).find())) {
            return "test";
        }
        return "feat";
    }

    private static String summary(List<PendingChange> changes) {
        long added = count(changes, PendingChange.Kind.ADDED);
        long modified = count(changes, PendingChange.Kind.MODIFIED);
        long deleted = count(changes, PendingChange.Kind.DELETED);
        long renamed = count(changes, PendingChange.Kind.RENAMED);
        StringBuilder line = new StringBuilder(changes.size() == 1 ? "1 file" : changes.size() + " files");
        String parts = String.join(", ", List.of(
                        part(added, "added"), part(modified, "modified"), part(deleted, "deleted"), part(renamed, "renamed"))
                .stream().filter(part -> !part.isEmpty()).toList());
        if (!parts.isEmpty()) {
            line.append(": ").append(parts);
        }
        return line.toString();
    }

    private static long count(List<PendingChange> changes, PendingChange.Kind kind) {
        return changes.stream().filter(change -> change.kind() == kind).count();
    }

    private static String part(long count, String label) {
        return count == 0 ? "" : count + " " + label.toLowerCase(Locale.ROOT);
    }

    private static String marker(PendingChange.Kind kind) {
        return switch (kind) {
            case ADDED -> "+";
            case MODIFIED -> "~";
            case DELETED -> "-";
            case RENAMED -> ">";
        };
    }
}
