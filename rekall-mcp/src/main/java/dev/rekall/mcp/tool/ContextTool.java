package dev.rekall.mcp.tool;

import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.ContextRecord;
import dev.rekall.domain.context.ContextService;
import dev.rekall.domain.context.DocumentView;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.step.TaskStepView;
import dev.rekall.domain.wrapup.WrapupView;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.protocol.ToolSchema;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;
import tools.jackson.databind.JsonNode;

import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;

/**
 * The only tool. Loads the working context named by a list of anchors, in one call.
 *
 * <p>Everything here is rendering: resolution and assembly belong to {@code ContextService},
 * which is the only thing that touches the database. What is left is turning a materialised
 * record into markdown, because the consumer is a model reading a context window rather than a
 * parser, and markdown says the same thing in fewer tokens.
 */
@Component
@RequiredArgsConstructor
public class ContextTool implements McpTool {

    /** Long notes are cut, and the cut is announced rather than silent. */
    private static final int MAX_DOCUMENT_CHARACTERS = 20_000;

    private final ContextService context;

    @Override
    public String name() {
        return "rekall_context";
    }

    @Override
    public String description() {
        return """
               Load the full working context for one or more anchors in a single call.

               An anchor is `entity:value`, for example `project:vega task:report-builder`. The
               entities are `company`, `project` and `task`.

               The value is the record's label, not its title. A label is lowercase, has no
               spaces, and is unique inside its parent: `project:vega`, never
               `project:"Vega Platform"`. The title is what the record is called and is free to
               change; the label is what this tool resolves.

               A bare term with no `entity:` is looked up across all three and accepted only when
               exactly one record matches; otherwise the candidates come back and nothing loads.

               A note can be attached to several tasks, so the same markdown may arrive under
               more than one anchor. That is deliberate: it is written once and read wherever
               it applies.

               A task also carries its wrapup, if it has one: what its implementation looks
               like now, written at the end of the last session. Read it as the current state of
               the work, not as a history of it. Replacing it is `rekall_wrapup`.

               A task may also carry a checklist of steps, and it is what says where the work
               has got to. An open step is the work: it arrives with its detail and it is what
               the session is for. A done step arrives as its title alone, because it is
               finished: do not build it again, and do not treat the missing detail as
               something to go and find. Read the open ones as the list of what is left, in
               order, and if the checklist and the wrapup disagree, the checklist is the one a
               person ticked.

               A done step marked `(finished since the wrapup was written)` is work the wrapup
               predates and cannot mention. It arrives with its detail for that reason alone,
               and it is what the next `rekall_wrapup` has to fold in: nothing else here says
               what that piece was.

               Where a task has open steps, they are the work and its description is not. The
               description is the standing context to build a step against: what the task is
               for, what the work has to satisfy, what is out of scope. It is written once and
               does not shrink as the work is done, so it goes on naming things that are
               already built, and reading it as a list of instructions is how the same thing
               gets built twice.

               Nothing you can call changes a step. They are ticked by hand in the console, by
               the person who reviewed the work.

               Each anchor returns the record, what it references resolved in full with their
               notes, what references it as anchors you can pass back, and all of its markdown.
               """;
    }

    @Override
    public Map<String, Object> inputSchema() {
        return ToolSchema.object()
                .requiredString(
                        "anchors",
                        "Space-separated anchors, e.g. `project:vega task:report-builder`. The value "
                                + "is the record's label, lowercase and without spaces, never its title. "
                                + "A single anchor is valid and loads that record alone.")
                .build();
    }

    @Override
    public String execute(JsonNode arguments) {
        List<Anchor> anchors = Anchor.parseAll(Arguments.of(arguments).requiredString("anchors"));
        StringBuilder out = new StringBuilder("# Context\n");

        // Anchoring a project and one of its tasks reaches the project twice, once directly and
        // once as the task's reference. Rendering it twice would spend the window saying the
        // same thing, so a record is written the first time it is reached and named after that.
        Set<String> rendered = new LinkedHashSet<>();
        for (ContextRecord record : load(anchors)) {
            out.append('\n').append(render(record, 2, rendered));
        }
        return out.toString();
    }

    /**
     * The two-anchor {@code project:x task:y} form resolves the task inside its project, which
     * is what disambiguates a task name that two projects both use. Any other combination is
     * resolved anchor by anchor.
     */
    private List<ContextRecord> load(List<Anchor> anchors) {
        try {
            if (anchors.size() == 2 && anchors.get(0).is("project") && anchors.get(1).is("task")) {
                return List.of(
                        context.load("project", anchors.get(0).value()),
                        context.loadTask(anchors.get(0).value(), anchors.get(1).value()));
            }
            return anchors.stream()
                    .map(anchor -> context.load(anchor.entityName(), anchor.value()))
                    .toList();
        } catch (UnknownAnchorException e) {
            throw new ToolFailure(e.getMessage());
        } catch (AmbiguousAnchorException e) {
            throw new ToolFailure(e.getMessage() + ". Qualify it as `entity:value`.");
        }
    }

    // ------------------------------------------------------------------ rendering

    private String render(ContextRecord record, int headingLevel, Set<String> rendered) {
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

    /**
     * The checklist, between the brief and the state, and asymmetric on purpose.
     *
     * <p>An open step is rendered with its detail because it is the work about to be done. A
     * done step is rendered as one line, because what it says now is only that it exists and
     * needs no doing: spending the window on the detail of finished work is how a context load
     * costs twice what it is worth and invites the same thing to be built again.
     *
     * <p>Ahead of the wrapup and after the description, which is the order the three are asked
     * in: what is this, what is left, and what did it become.
     */
    private String renderSteps(List<TaskStepView> steps, WrapupView wrapup) {
        if (steps.isEmpty()) {
            return "";
        }
        long done = steps.stream().filter(TaskStepView::done).count();
        long unwritten = steps.stream().filter(step -> isUnwritten(step, wrapup)).count();

        StringBuilder out = new StringBuilder("\n<steps done=\"%d\" open=\"%d\"%s>\n".formatted(
                done,
                steps.size() - done,
                unwritten == 0 ? "" : " finished-since-wrapup=\"%d\"".formatted(unwritten)));

        for (TaskStepView step : steps) {
            boolean unwrittenHere = isUnwritten(step, wrapup);
            out.append(step.done() ? "- [x] " : "- [ ] ").append(step.title());
            if (unwrittenHere) {
                out.append("  (finished since the wrapup was written)");
            }
            out.append('\n');

            // The detail comes back for a step the wrapup has not caught up with. It is done, so
            // nothing is left to build from it, but nothing else in this context says what it
            // was: the wrapup predates it and the title alone is not enough to describe what the
            // task now is. It goes silent again as soon as a wrapup accounts for it.
            boolean carriesDetail = !step.done() || unwrittenHere;
            if (carriesDetail && step.bodyMarkdown() != null && !step.bodyMarkdown().isBlank()) {
                out.append(indent(truncate(step.bodyMarkdown()))).append('\n');
            }
        }
        return out.append("</steps>\n").toString();
    }

    /**
     * Whether this step finished after the wrapup was last written.
     *
     * <p>The one thing a model cannot work out for itself here, and the reason it is computed
     * rather than left to be inferred from two timestamps in the text. A step ticked after the
     * wrapup is a piece of work the current description cannot possibly mention, which is
     * exactly what the next wrapup has to fold in. With no wrapup at all, every finished step
     * is unaccounted for, and the first one written has to cover them all.
     */
    private boolean isUnwritten(TaskStepView step, WrapupView wrapup) {
        if (!step.done() || step.doneAt() == null) {
            return false;
        }
        return wrapup == null || step.doneAt().isAfter(wrapup.updatedAt());
    }

    /** Two spaces, so a step's detail stays inside the item it belongs to rather than ending it. */
    private String indent(String body) {
        return body.lines().map(line -> line.isBlank() ? "" : "  " + line).collect(Collectors.joining("\n"));
    }

    /**
     * The wrapup, ahead of the notes, in a tag of its own.
     *
     * <p>Ahead because a session that opens on a task is asking what it does now, and the notes
     * are the background to that answer rather than the answer. In its own tag because it is not
     * a note and must not be edited as one: it is replaced whole, by `rekall_wrapup`.
     */
    private String renderWrapup(WrapupView wrapup) {
        if (wrapup == null) {
            return "";
        }
        return "\n<wrapup written-by=\"%s\" updated=\"%s\">\n%s\n</wrapup>\n"
                .formatted(wrapup.writtenBy(), wrapup.updatedAt(), truncate(wrapup.bodyMarkdown()));
    }

    /**
     * What the record is, in a tag rather than in the bullet list above it.
     *
     * <p>A description is a markdown document, saying what the work is, what it has to satisfy and
     * what is out of scope, and a document inlined into a list item stops being one: every line after
     * the first falls outside the bullet, and its own headings outrank the record's.
     */
    private String renderDescription(String description) {
        if (description == null || description.isBlank()) {
            return "";
        }
        return "\n<description>\n" + truncate(description) + "\n</description>\n";
    }

    private String renderBlueprint(String blueprint) {
        if (blueprint == null || blueprint.isBlank()) {
            return "";
        }
        return "\n<blueprint>\n" + truncate(blueprint) + "\n</blueprint>\n";
    }

    private String renderDocuments(List<DocumentView> documents) {
        if (documents.isEmpty()) {
            return "";
        }
        StringBuilder out = new StringBuilder();
        for (DocumentView document : documents) {
            out.append("\n<document title=\"").append(document.title())
                    .append("\" kind=\"").append(document.kind()).append("\">\n")
                    .append(truncate(document.bodyMarkdown()))
                    .append("\n</document>\n");
        }
        return out.toString();
    }

    private String truncate(String body) {
        if (body == null) {
            return "";
        }
        if (body.length() <= MAX_DOCUMENT_CHARACTERS) {
            return body;
        }
        return body.substring(0, MAX_DOCUMENT_CHARACTERS)
                + "\n\n[truncated: %d of %d characters shown]".formatted(MAX_DOCUMENT_CHARACTERS, body.length());
    }
}
