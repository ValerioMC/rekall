package dev.rekall.mcp.tool;

import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.note.NoteService;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.protocol.ToolSchema;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;
import tools.jackson.databind.JsonNode;

import java.util.Map;

/**
 * Lets a session keep what it produced as a note on the task it is working, rather than folding
 * it into a wrapup that is not the place for it. It only ever adds a note: nothing already in
 * Rekall can be edited, detached or deleted through it.
 */
@Component
@RequiredArgsConstructor
public class NoteTool implements McpTool {

    private final NoteService notes;

    @Override
    public String name() {
        return "rekall_note";
    }

    @Override
    public boolean writes() {
        return true;
    }

    @Override
    public String description() {
        return """
               Write a new markdown note and attach it to one task. It loads with that task's \
               context from then on, in full, and a person can edit it, share it with other \
               tasks or delete it in the console.

               Use it when the work produces something worth keeping that is not the state of \
               the implementation: the output a step or the description asks for (an analysis, \
               a list, a draft, a runbook), a reference someone will want again, or a note the \
               task exists to write. It is not the wrapup: what the implementation looks like \
               now goes to `rekall_wrapup`, and a note does not replace it.

               - `title`: what the note is, short, the way a file would be named \
               ("export-formats.md").
               - `body`: the whole note, in markdown, written to be read on its own by someone \
               who was not in this session.

               It only adds. A title the task's notes already carry is refused rather than \
               overwritten, and nothing here can change a note that is already there.

               Anchor the task the way `rekall_context` does: `project:vega task:report-builder`.
               """;
    }

    @Override
    public Map<String, Object> inputSchema() {
        return ToolSchema.object()
                .requiredString(
                        "anchors",
                        "The task to attach the note to, e.g. `project:vega task:report-builder`. Labels, "
                                + "never titles, and it has to name exactly one task.")
                .requiredString("title", "What the note is, short, like a file name.")
                .requiredString("body", "The complete note, in markdown.")
                .build();
    }

    @Override
    public String execute(JsonNode arguments) {
        Arguments args = Arguments.of(arguments);
        AnchoredTask target = AnchoredTask.from(Anchor.parseAll(args.requiredString("anchors")));
        NoteService.Written written;
        try {
            written = notes.write(
                    target.projectLabel(), target.taskLabel(), args.requiredString("title"), args.requiredString("body"));
        } catch (UnknownAnchorException e) {
            throw new ToolFailure(e.getMessage());
        } catch (AmbiguousAnchorException e) {
            throw new ToolFailure(e.getMessage() + ". Qualify it with `project:<label>`.");
        } catch (IllegalArgumentException e) {
            throw new ToolFailure(e.getMessage());
        }
        return """
               Note "%s" written on `%s` as `%s`. The task carries %d note%s, and `/rk %s` loads \
               this one with it from now on.\
               """.formatted(
                written.title(), written.taskAnchor(), written.noteAnchor(), written.notesOnTask(),
                written.notesOnTask() == 1 ? "" : "s", written.taskAnchor());
    }
}
