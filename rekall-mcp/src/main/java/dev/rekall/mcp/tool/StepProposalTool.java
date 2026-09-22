package dev.rekall.mcp.tool;

import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.step.TaskStepService;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.protocol.ToolSchema;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;
import tools.jackson.databind.JsonNode;

import java.util.Map;

/**
 * Lets a planning session put the checklist it proposes on the task, as drafts. It can only ever
 * create a draft: nothing it writes is work until a person promotes it in the console, so the
 * read-mostly boundary holds.
 */
@Component
@RequiredArgsConstructor
public class StepProposalTool implements McpTool {

    private final TaskStepService steps;

    @Override
    public String name() {
        return "rekall_propose_step";
    }

    @Override
    public boolean writes() {
        return true;
    }

    @Override
    public String description() {
        return """
               Propose one step for a task's checklist. It lands as a draft on the staging shelf \
               in the console, never on the checklist itself: a person reads it, rewords it if \
               they want, and promotes it or deletes it. Until then no session sees it as work \
               and `rekall_step` will not move it.

               Use it when you were asked to plan a task. Read the task with `rekall_context` \
               first: the description says what the work is for and what it has to satisfy, the \
               wrapup says what already exists, and the code says the rest. Propose only what is \
               still missing, one call per step, in the order the work should be done.

               - `title`: what the step delivers, as a short imperative line ("Expose the report \
               as a download"), under 200 characters.
               - `detail`: markdown. What to build, where it goes in the code, what it must \
               satisfy, and how a reviewer can tell it is done. Enough for a session that never \
               saw this one to do the step alone.

               A title the task already has is refused, so running a plan again does not \
               duplicate it. A task holds at most 20 drafts at once.

               Anchor the task the way `rekall_context` does: `project:vega task:report-builder`.
               """;
    }

    @Override
    public Map<String, Object> inputSchema() {
        return ToolSchema.object()
                .requiredString(
                        "anchors",
                        "The task to propose the step for, e.g. `project:vega task:report-builder`. Labels, "
                                + "never titles, and it has to name exactly one task.")
                .requiredString("title", "What the step delivers, as a short imperative line.")
                .optionalString(
                        "detail",
                        "Markdown: what to build, where, what it must satisfy, and how a reviewer can tell "
                                + "it is done.")
                .build();
    }

    @Override
    public String execute(JsonNode arguments) {
        Arguments args = Arguments.of(arguments);
        AnchoredTask target = AnchoredTask.from(Anchor.parseAll(args.requiredString("anchors")));
        TaskStepService.Proposed proposed;
        try {
            proposed = steps.propose(
                    target.projectLabel(), target.taskLabel(), args.requiredString("title"), args.optionalString("detail"));
        } catch (UnknownAnchorException e) {
            throw new ToolFailure(e.getMessage());
        } catch (AmbiguousAnchorException e) {
            throw new ToolFailure(e.getMessage() + ". Qualify it with `project:<label>`.");
        } catch (IllegalArgumentException e) {
            throw new ToolFailure(e.getMessage());
        }
        return """
               Draft "%s" proposed. The task holds %d draft%s waiting on the console's staging \
               shelf; none of them is work until a person promotes it.\
               """.formatted(
                proposed.step().title(), proposed.draftCount(), proposed.draftCount() == 1 ? "" : "s");
    }
}
