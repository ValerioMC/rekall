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

import java.util.Map;

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

               Name the code, do not transcribe it. It is read next to the repository, so say \
               where things are by the names they have: the class, the file, the endpoint, the \
               table, the component, a line or two each on what it is for and what it decides.

               - Yes: what an entity is there for, and, where there is business logic, the rule \
               itself: what it decides, on what, what happens at the edges, what is refused.
               - No: the fields of an object, the columns of a table, method signatures, \
               parameter lists, a directory tree. That is in the code, it is longer than the \
               wrapup, and it is wrong a week later.

               Small enough to read in one go. Short paragraphs or short bullets, not an essay \
               and not an index.

               Read the current wrapup with `rekall_context` before replacing it: it arrives \
               with the task. Keep what is still true and rewrite the rest, rather than \
               describing only the piece you touched.

               Written after a step was finished, it has to fold that step's work into the \
               same description rather than append to it. `rekall_context` marks every finished \
               step this wrapup predates, and all of them belong in the new text, not only the \
               one this session closed: a step ticked in an earlier session that never got a \
               wrapup is still waiting to be described. One account of the whole task, with \
               the new piece named where it lives; never a section per step, and never a \
               heading carrying a step's title. If the step made something the wrapup already \
               said untrue, that sentence goes.
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
        AnchoredTask target = AnchoredTask.from(Anchor.parseAll(args.requiredString("anchors")));
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

        if (written.replaced() == WrapupAuthor.HAND) {
            out.append("\nThe version you replaced had been edited by hand in the console. ")
                    .append("If that edit said something this one does not, it is gone.\n");
        }

        return out.append("\nIt is what `/rk ")
                .append(written.wrapup().anchor())
                .append("` will load from now on.")
                .toString();
    }
}
