package dev.rekall.domain.context;

import dev.rekall.domain.DocumentContextMode;
import dev.rekall.domain.TaskStepState;
import dev.rekall.domain.step.TaskStepView;
import dev.rekall.domain.wrapup.WrapupView;
import org.springframework.stereotype.Component;

import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

/**
 * Turns loaded context records into the markdown a session reads. It is the one renderer, so what
 * {@code rekall_context} hands over and what the console measures as the size of a task's context
 * are the same text.
 */
@Component
public class ContextRenderer {

    static final int MAX_DOCUMENT_CHARACTERS = 20_000;

    /** A diff is the one thing here worth more room than a note: it is what the session was handed to read. */
    static final int MAX_DIFF_CHARACTERS = 60_000;

    /** How much of a reference note's opening line travels with it. */
    static final int REFERENCE_LINE_MAX = 200;

    /** The whole answer to one {@code rekall_context} call: every record once, in the order loaded. */
    public String render(List<ContextRecord> records) {
        StringBuilder out = new StringBuilder("# Context\n");
        Set<String> rendered = new LinkedHashSet<>();
        for (ContextRecord record : records) {
            out.append('\n').append(render(record, 2, rendered));
        }
        return out.toString();
    }

    String render(ContextRecord record, int headingLevel, Set<String> rendered) {
        if (!rendered.add(record.anchor())) {
            return "";
        }
        String heading = "#".repeat(headingLevel);
        StringBuilder out = new StringBuilder(heading)
                .append(' ')
                .append(record.kind())
                .append(": ")
                .append(record.label())
                .append("\n\n")
                .append("- `anchor`: `").append(record.anchor()).append("`\n");

        record.fields().forEach((key, value) -> out.append("- `").append(key).append("`: ").append(value).append('\n'));

        if (!record.related().isEmpty()) {
            out.append("- `related`:\n");
            record.related().forEach(anchor -> out.append("  - `").append(anchor).append("`\n"));
        }

        out.append(renderDescription(record.description()));
        out.append(renderBlueprint(record.blueprint()));
        out.append(renderSteps(record.steps(), record.wrapup()));
        out.append(renderCommits(record.commits()));
        out.append(renderWrapup(record.wrapup()));
        out.append(renderDocuments(record.documents()));
        for (ContextRecord reference : record.references()) {
            String nested = render(reference, headingLevel + 1, rendered);
            if (!nested.isEmpty()) {
                out.append('\n').append(nested);
            }
        }
        return out.toString();
    }

    String renderSteps(List<TaskStepView> allSteps, WrapupView wrapup) {
        long drafts = allSteps.stream().filter(step -> step.state().draft()).count();
        List<TaskStepView> steps = allSteps.stream().filter(step -> !step.state().draft()).toList();
        if (steps.isEmpty()) {
            return "";
        }
        long done = steps.stream().filter(step -> step.state().complete()).count();
        long running = steps.stream().filter(step -> step.state().running()).count();
        long awaiting = steps.stream().filter(step -> step.state() == TaskStepState.CLAIMED).count();
        long unwritten = steps.stream().filter(step -> isUnwritten(step, wrapup)).count();

        StringBuilder attributes = new StringBuilder("done=\"%d\" open=\"%d\"".formatted(done, steps.size() - done));
        if (running > 0) {
            attributes.append(" running=\"%d\"".formatted(running));
        }
        if (awaiting > 0) {
            attributes.append(" awaiting-review=\"%d\"".formatted(awaiting));
        }
        if (unwritten > 0) {
            attributes.append(" finished-since-wrapup=\"%d\"".formatted(unwritten));
        }
        if (drafts > 0) {
            attributes.append(" draft=\"%d\"".formatted(drafts));
        }
        StringBuilder out = new StringBuilder("\n<steps ").append(attributes).append(">\n");

        for (TaskStepView step : steps) {
            boolean unwrittenHere = isUnwritten(step, wrapup);
            out.append(step.state().complete() ? "- [x] " : "- [ ] ").append(step.title());
            if (step.state().running()) {
                out.append("  (in progress)");
            } else if (step.state() == TaskStepState.CLAIMED) {
                out.append("  (claimed, waiting for the console to accept it)");
            }
            if (unwrittenHere) {
                out.append("  (finished since the wrapup was written)");
            }
            out.append('\n');

            boolean carriesDetail = !step.state().complete() || unwrittenHere;
            if (carriesDetail && step.bodyMarkdown() != null && !step.bodyMarkdown().isBlank()) {
                out.append(indent(truncate(step.bodyMarkdown()))).append('\n');
            }
        }
        if (drafts > 0) {
            out.append("<!-- %d further step%s still in draft, not shown: promoted to the checklist "
                    .formatted(drafts, drafts == 1 ? "" : "s"))
                    .append("in the console when it is ready to work -->\n");
        }
        return out.append("</steps>\n").toString();
    }

    private boolean isUnwritten(TaskStepView step, WrapupView wrapup) {
        if (!step.state().complete() || step.completedAt() == null) {
            return false;
        }
        return wrapup == null || step.completedAt().isAfter(wrapup.updatedAt());
    }

    private String indent(String body) {
        return body.lines().map(line -> line.isBlank() ? "" : "  " + line).collect(Collectors.joining("\n"));
    }

    String renderWrapup(WrapupView wrapup) {
        if (wrapup == null) {
            return "";
        }
        return "\n<wrapup written-by=\"%s\" updated=\"%s\">\n%s\n</wrapup>\n"
                .formatted(wrapup.writtenBy(), wrapup.updatedAt(), truncate(wrapup.bodyMarkdown()));
    }

    String renderCommits(List<ContextCommitView> commits) {
        if (commits.isEmpty()) {
            return "";
        }
        StringBuilder out = new StringBuilder("\n<commits count=\"%d\">\n".formatted(commits.size()));
        for (ContextCommitView commit : commits) {
            out.append("<commit hash=\"").append(commit.commitHash()).append('"');
            if (commit.stepTitle() != null) {
                out.append(" step=\"").append(commit.stepTitle().replace("\"", "'")).append('"');
            }
            out.append(">\n").append(commit.comment()).append('\n');
            if (commit.diff() == null || commit.diff().isBlank()) {
                out.append("\n(no diff was recorded for this commit; read it from the repository by its hash)\n");
            } else {
                out.append('\n').append(truncate(commit.diff(), MAX_DIFF_CHARACTERS)).append('\n');
            }
            out.append("</commit>\n");
        }
        return out.append("</commits>\n").toString();
    }

    String renderDescription(String description) {
        if (description == null || description.isBlank()) {
            return "";
        }
        return "\n<description>\n" + truncate(description) + "\n</description>\n";
    }

    String renderBlueprint(String blueprint) {
        if (blueprint == null || blueprint.isBlank()) {
            return "";
        }
        return "\n<blueprint>\n" + truncate(blueprint) + "\n</blueprint>\n";
    }

    String renderDocuments(List<DocumentView> documents) {
        if (documents.isEmpty()) {
            return "";
        }
        StringBuilder out = new StringBuilder();
        for (DocumentView document : documents) {
            out.append(document.contextMode() == DocumentContextMode.REFERENCE
                    ? renderReference(document)
                    : "\n<document title=\"%s\" kind=\"%s\">\n%s\n</document>\n"
                            .formatted(document.title(), document.kind(), truncate(document.bodyMarkdown())));
        }
        return out.toString();
    }

    /**
     * A reference note: its title, its opening line so the session can judge whether it applies,
     * and the anchor that loads the rest. The body stays out of the context until it is asked for.
     */
    String renderReference(DocumentView document) {
        return ("\n<document title=\"%s\" kind=\"%s\" anchor=\"%s\" loaded=\"on request\">\n%s\n"
                + "Not included. If the work needs it, load it with `rekall_context` and the anchor `%s`.\n"
                + "</document>\n").formatted(
                document.title(), document.kind(), document.anchor(), openingLine(document.bodyMarkdown()),
                document.anchor());
    }

    private static String openingLine(String body) {
        String first = body == null ? "" : body.lines()
                .map(String::strip)
                .filter(line -> !line.isEmpty() && !line.startsWith("#") && !line.startsWith("```"))
                .findFirst()
                .orElse("");
        return first.length() <= REFERENCE_LINE_MAX ? first : first.substring(0, REFERENCE_LINE_MAX).stripTrailing() + "…";
    }

    private String truncate(String body) {
        return truncate(body, MAX_DOCUMENT_CHARACTERS);
    }

    private String truncate(String body, int limit) {
        if (body == null) {
            return "";
        }
        if (body.length() <= limit) {
            return body;
        }
        return body.substring(0, limit)
                + "\n\n[truncated: %d of %d characters shown]".formatted(limit, body.length());
    }
}
