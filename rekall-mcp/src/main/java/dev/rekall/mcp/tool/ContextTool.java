package dev.rekall.mcp.tool;

import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.ContextRecord;
import dev.rekall.domain.context.ContextService;
import dev.rekall.domain.context.DocumentView;
import dev.rekall.domain.TaskStepState;
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

@Component
@RequiredArgsConstructor
public class ContextTool implements McpTool {

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

               A draft step is not in the checklist you get. It is a line the person is still
               wording, counted only as `draft="N"` on the `<steps>` tag. It carries no work
               yet, `rekall_step` will not move it, and it becomes a real step when the console
               promotes it.

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

        Set<String> rendered = new LinkedHashSet<>();
        for (ContextRecord record : load(anchors)) {
            out.append('\n').append(render(record, 2, rendered));
        }
        return out.toString();
    }

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

    private String renderSteps(List<TaskStepView> allSteps, WrapupView wrapup) {
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

    private String renderWrapup(WrapupView wrapup) {
        if (wrapup == null) {
            return "";
        }
        return "\n<wrapup written-by=\"%s\" updated=\"%s\">\n%s\n</wrapup>\n"
                .formatted(wrapup.writtenBy(), wrapup.updatedAt(), truncate(wrapup.bodyMarkdown()));
    }

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
