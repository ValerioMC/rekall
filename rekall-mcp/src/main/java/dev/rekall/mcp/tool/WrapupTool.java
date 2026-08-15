package dev.rekall.mcp.tool;

import dev.rekall.domain.WrapupAuthor;
import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.wrapup.WrapupService;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.protocol.ToolSchema;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;
import tools.jackson.databind.JsonNode;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

/**
 * The one tool that writes: it replaces a task's wrapup.
 *
 * <p>A wrapup is what the task's implementation looks like now. It is written at the end of a
 * session and read at the start of the next one, which is the loop the whole feature exists
 * for: the work resumes from the current state instead of from reading the code back.
 *
 * <p>Everything this can reach is one row of one table, keyed by a task. It cannot create a
 * task, rename one, touch a note or write any other column, and there is deliberately no tool
 * beside it: a second way to write is a second thing to get wrong.
 */
@Component
@RequiredArgsConstructor
public class WrapupTool implements McpTool {

    private final WrapupService wrapups;

    @Override
    public String name() {
        return "rekall_wrapup";
    }

    @Override
    public boolean writes() {
        return true;
    }

    @Override
    public String description() {
        return """
               Record what a task's implementation looks like now. One task, one wrapup, \
               replaced in place.

               Anchor the task the way `rekall_context` does: `project:vega task:report-builder`. \
               A bare `task:<label>` works when that label exists on only one project.

               `body` is the complete new text and it overwrites whatever was there. Nothing is \
               merged and no previous version is kept, so send the wrapup in full every time, \
               not the part that changed.

               Write the state, never the story of getting there. It describes the system as it \
               stands, for a reader who was not in this session and does not care what it looked \
               like before:

               - Yes: what exists, what it does, how the parts fit, what is settled and what is \
               still open, where the sharp edges are.
               - No: "added", "changed", "now also", "previously", "fixed", "refactored", \
               "before/after", anything dated, anything phrased as a step you took.

               If a sentence only makes sense to someone who watched the change happen, it does \
               not belong in a wrapup. Notes are where the reasoning and the history go.

               Read the current wrapup with `rekall_context` before replacing it: it arrives \
               with the task. Keep what is still true and rewrite the rest, rather than \
               describing only the piece you touched.
               """;
    }

    @Override
    public Map<String, Object> inputSchema() {
        return ToolSchema.object()
                .requiredString(
                        "anchors",
                        "The task to write to, e.g. `project:vega task:report-builder`. Labels, never "
                                + "titles. It has to name exactly one task; a `company:` anchor cannot.")
                .requiredString(
                        "body",
                        "The complete wrapup, in markdown. Replaces what was there. Describes the "
                                + "implementation as it stands, not what changed in this session.")
                .build();
    }

    @Override
    public String execute(JsonNode arguments) {
        Arguments args = Arguments.of(arguments);
        Target target = target(Anchor.parseAll(args.requiredString("anchors")));
        String body = args.requiredString("body");

        WrapupService.Written written;
        try {
            written = wrapups.write(target.projectLabel(), target.taskLabel(), body, WrapupAuthor.CLAUDE);
        } catch (UnknownAnchorException e) {
            throw new ToolFailure(e.getMessage());
        } catch (AmbiguousAnchorException e) {
            throw new ToolFailure(e.getMessage() + ". Qualify it with `project:<label>`.");
        } catch (IllegalArgumentException e) {
            throw new ToolFailure(e.getMessage());
        }

        StringBuilder out = new StringBuilder(written.created() ? "Wrapup written for " : "Wrapup replaced for ")
                .append('`').append(written.wrapup().anchor()).append("`.\n");

        // The one outcome worth saying out loud. A wrapup corrected in the console is the user
        // disagreeing with the last one written here, and overwriting that silently is how the
        // correction gets lost twice.
        if (written.replaced() == WrapupAuthor.HAND) {
            out.append("\nThe version you replaced had been edited by hand in the console. ")
                    .append("If that edit said something this one does not, it is gone.\n");
        }

        return out.append("\nIt is what `/rk ")
                .append(written.wrapup().anchor())
                .append("` will load from now on.")
                .toString();
    }

    // ------------------------------------------------------------------ addressing

    /**
     * Which task the anchors name.
     *
     * @param projectLabel the {@code project:} half, or null when only a task was given
     */
    private record Target(String projectLabel, String taskLabel) {
    }

    /**
     * A write has to land on one task and be sure of it.
     *
     * <p>The reading tool can afford to accept a company or a project anchor and hand back
     * whatever hangs off it. This one cannot: a project names forty tasks and none of them is
     * the answer, so anything that does not resolve to a single task is refused with the form
     * that would have worked.
     */
    private Target target(List<Anchor> anchors) {
        String project = null;
        String task = null;
        List<String> bare = new ArrayList<>();

        for (Anchor anchor : anchors) {
            if (anchor.is("task")) {
                if (task != null) {
                    throw new ToolFailure("Two task anchors were given. A wrapup belongs to one task.");
                }
                task = anchor.value();
            } else if (anchor.is("project")) {
                project = anchor.value();
            } else if (!anchor.isQualified()) {
                bare.add(anchor.value());
            } else {
                throw new ToolFailure(
                        "`%s` cannot say which task to write to. Pass `project:<label> task:<label>`."
                                .formatted(anchor));
            }
        }

        // One bare term is the positional form of a task label, the same way `/rk report-builder`
        // reads. More than one is a guess, and this is the wrong place to guess.
        if (task == null && bare.size() == 1) {
            task = bare.getFirst();
        }
        if (task == null) {
            throw new ToolFailure(
                    "No task in those anchors. A wrapup belongs to exactly one task: "
                            + "pass `project:<label> task:<label>`.");
        }
        return new Target(project, task);
    }
}
