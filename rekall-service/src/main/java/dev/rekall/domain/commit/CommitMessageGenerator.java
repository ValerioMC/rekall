package dev.rekall.domain.commit;

import java.util.ArrayList;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Writes the message for an automatic commit: a Conventional Commits subject and a prose body
 * that says what the change does, never which files it touched.
 *
 * <p>The words come from the best source there is. A session that claims work can hand over its
 * own message, and that wins: its first line is the subject, the rest is the body. Without one,
 * the subject is the title of what was claimed and the body is drawn from the text that describes
 * it, the step's detail or the task's wrapup, turned from markdown into plain paragraphs and cut
 * to a few sentences. A final {@code Refs:} line names the task and step either way.
 *
 * <p>The type is the session's when it wrote one. Otherwise the title decides {@code fix} and
 * {@code refactor}, and the files decide {@code docs} (every file is documentation) and
 * {@code test} (every file is a test); anything else is {@code feat}.
 */
final class CommitMessageGenerator {

    /** Git shows the subject in full up to here; past it, tools wrap or cut. */
    static final int SUBJECT_MAX = 72;

    /** Body lines are wrapped here, the width {@code git log} is read at. */
    static final int BODY_WIDTH = 72;

    /** How much of a step's detail or a wrapup the derived body carries. */
    static final int DERIVED_BODY_MAX = 800;

    private static final Pattern DOCUMENTATION = Pattern.compile(
            "(?i)(^|/)(docs?|documentation)/|\\.(md|mdx|markdown|rst|txt|adoc)$");

    private static final Pattern TEST = Pattern.compile(
            "(?i)(^|/)(tests?|__tests__|spec|e2e)/|\\.(spec|test)\\.[a-z]+$|Tests?\\.java$|_test\\.(go|py)$|test_[^/]*\\.py$");

    private static final Pattern CONVENTIONAL_PREFIX = Pattern.compile(
            "^(feat|fix|docs|test|refactor|chore|perf|build|ci|style|revert)(\\([^)]*\\))?!?: .+");

    private static final Pattern FIX_TITLE = Pattern.compile(
            "(?iu)^(fix|bug|hotfix|correggi|corregge|correzione|risolvi|risolve|errore|problema)\\b");

    private static final Pattern REFACTOR_TITLE = Pattern.compile("(?iu)^(refactor|refactoring|riorganizza|ripulisci)\\b");

    private static final Pattern LIST_ITEM = Pattern.compile("^(\\s*)([-*+]|\\d+[.)])\\s+(.*)$");

    private static final Pattern SENTENCE_END = Pattern.compile("[.!?](\\s|$)");

    /**
     * What the commit is for: the step or task claimed, as a title, where it lives, and the
     * markdown that describes it (a step's detail or the task's wrapup), which may be absent.
     */
    record Subject(String title, String anchor, Integer stepNumber, String description) {
    }

    private CommitMessageGenerator() {
    }

    /**
     * @param sessionMessage the message the claiming session wrote, or {@code null} to derive one
     */
    static String generate(Subject subject, List<PendingChange> changes, String sessionMessage) {
        SessionMessage session = SessionMessage.parse(sessionMessage);
        String subjectLine = session.subject() == null
                ? subjectLine(subject, changes)
                : sessionSubjectLine(session.subject(), subject, changes);
        String body = session.body() == null ? derivedBody(subject.description()) : session.body();

        StringBuilder message = new StringBuilder(subjectLine).append("\n\n");
        if (!body.isBlank()) {
            message.append(wrap(body)).append("\n\n");
        }
        message.append("Refs: ").append(subject.anchor());
        if (subject.stepNumber() != null) {
            message.append(", step ").append(subject.stepNumber());
        }
        return message.append('\n').toString();
    }

    static String subjectLine(Subject subject, List<PendingChange> changes) {
        return fitted(typeOf(subject.title(), changes) + ": ", collapsed(subject.title()));
    }

    static String typeOf(String title, List<PendingChange> changes) {
        String plain = collapsed(title);
        if (FIX_TITLE.matcher(plain).find()) {
            return "fix";
        }
        if (REFACTOR_TITLE.matcher(plain).find()) {
            return "refactor";
        }
        if (!changes.isEmpty() && changes.stream().allMatch(change -> DOCUMENTATION.matcher(change.path()).find())) {
            return "docs";
        }
        if (!changes.isEmpty() && changes.stream().allMatch(change -> TEST.matcher(change.path()).find())) {
            return "test";
        }
        return "feat";
    }

    private static String sessionSubjectLine(String written, Subject subject, List<PendingChange> changes) {
        String line = collapsed(written);
        if (CONVENTIONAL_PREFIX.matcher(line).matches()) {
            int colon = line.indexOf(": ");
            return fitted(line.substring(0, colon + 2), line.substring(colon + 2));
        }
        return fitted(typeOf(subject.title(), changes) + ": ", line);
    }

    private static String fitted(String prefix, String title) {
        int room = SUBJECT_MAX - prefix.length();
        if (title.length() > room) {
            title = title.substring(0, room - 1).stripTrailing() + "…";
        }
        return prefix + title;
    }

    /**
     * The body when the session wrote none: the opening paragraphs of the markdown that describes
     * the work, headings, emphasis and code fences dropped, cut at a paragraph or a sentence.
     */
    static String derivedBody(String markdown) {
        if (markdown == null || markdown.isBlank()) {
            return "";
        }
        List<String> paragraphs = plainParagraphs(markdown);
        StringBuilder body = new StringBuilder();
        for (String paragraph : paragraphs) {
            int separator = body.isEmpty() ? 0 : 2;
            if (body.length() + separator + paragraph.length() <= DERIVED_BODY_MAX) {
                if (separator > 0) {
                    body.append("\n\n");
                }
                body.append(paragraph);
                continue;
            }
            if (body.isEmpty()) {
                body.append(cutAtSentence(paragraph, DERIVED_BODY_MAX));
            }
            break;
        }
        return body.toString();
    }

    private static List<String> plainParagraphs(String markdown) {
        List<String> paragraphs = new ArrayList<>();
        List<String> current = new ArrayList<>();
        boolean inFence = false;
        for (String raw : markdown.strip().split("\\R")) {
            String line = raw.stripTrailing();
            if (line.strip().startsWith("```") || line.strip().startsWith("~~~")) {
                inFence = !inFence;
                continue;
            }
            if (inFence) {
                continue;
            }
            if (line.isBlank() || line.strip().startsWith("#") || line.strip().matches("^[-*_]{3,}$")) {
                flush(current, paragraphs);
                continue;
            }
            current.add(unemphasised(line));
        }
        flush(current, paragraphs);
        return paragraphs;
    }

    private static void flush(List<String> lines, List<String> paragraphs) {
        if (lines.isEmpty()) {
            return;
        }
        boolean list = lines.stream().anyMatch(line -> LIST_ITEM.matcher(line).matches());
        paragraphs.add(list
                ? String.join("\n", lines.stream().map(String::strip).toList())
                : collapsed(String.join(" ", lines)));
        lines.clear();
    }

    private static String unemphasised(String line) {
        return line.replace("**", "").replace("__", "").replaceAll("^\\s*>\\s?", "");
    }

    private static String cutAtSentence(String text, int max) {
        String head = text.substring(0, Math.min(text.length(), max));
        Matcher end = SENTENCE_END.matcher(head);
        int lastEnd = -1;
        while (end.find()) {
            lastEnd = end.start() + 1;
        }
        if (lastEnd > max / 3) {
            return head.substring(0, lastEnd);
        }
        int space = head.lastIndexOf(' ');
        return (space > 0 ? head.substring(0, space) : head).stripTrailing() + "…";
    }

    /**
     * Reflows the body at {@link #BODY_WIDTH}: the lines of a paragraph are joined and rewrapped,
     * a list item keeps its marker and a hanging indent, and blank lines between paragraphs stay.
     */
    static String wrap(String body) {
        List<String> out = new ArrayList<>();
        String marker = null;
        StringBuilder block = new StringBuilder();
        for (String line : body.strip().split("\\R", -1)) {
            Matcher item = LIST_ITEM.matcher(line);
            if (line.isBlank() || item.matches()) {
                flushBlock(block, marker, out);
                marker = null;
                if (line.isBlank()) {
                    out.add("");
                    continue;
                }
                marker = item.group(1) + item.group(2) + " ";
                block.append(item.group(3));
                continue;
            }
            if (!block.isEmpty()) {
                block.append(' ');
            }
            block.append(line.strip());
        }
        flushBlock(block, marker, out);
        return String.join("\n", out).replaceAll("\n{3,}", "\n\n");
    }

    private static void flushBlock(StringBuilder block, String marker, List<String> out) {
        if (block.isEmpty()) {
            return;
        }
        out.addAll(marker == null
                ? wrapLine(block.toString(), "", "")
                : wrapLine(block.toString(), marker, " ".repeat(marker.length())));
        block.setLength(0);
    }

    private static List<String> wrapLine(String text, String firstIndent, String nextIndent) {
        List<String> lines = new ArrayList<>();
        StringBuilder line = new StringBuilder(firstIndent);
        int contentStart = firstIndent.length();
        for (String word : collapsed(text).split(" ")) {
            if (word.isEmpty()) {
                continue;
            }
            boolean empty = line.length() == contentStart;
            if (!empty && line.length() + 1 + word.length() > BODY_WIDTH) {
                lines.add(line.toString());
                line = new StringBuilder(nextIndent);
                contentStart = nextIndent.length();
                empty = true;
            }
            if (!empty) {
                line.append(' ');
            }
            line.append(word);
        }
        lines.add(line.toString());
        return lines;
    }

    private static String collapsed(String text) {
        return text == null ? "" : text.strip().replaceAll("\\s+", " ");
    }

    /** A session's own message, split into a subject and a body; either may be absent. */
    private record SessionMessage(String subject, String body) {

        static SessionMessage parse(String message) {
            if (message == null || message.isBlank()) {
                return new SessionMessage(null, null);
            }
            String text = message.strip();
            int newline = text.indexOf('\n');
            if (newline < 0) {
                return new SessionMessage(text, null);
            }
            String body = text.substring(newline + 1).strip();
            return new SessionMessage(text.substring(0, newline), body.isEmpty() ? null : body);
        }
    }
}
