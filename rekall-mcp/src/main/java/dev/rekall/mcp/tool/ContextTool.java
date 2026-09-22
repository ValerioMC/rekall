package dev.rekall.mcp.tool;

import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.ContextRecord;
import dev.rekall.domain.context.ContextRenderer;
import dev.rekall.domain.context.ContextService;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.protocol.ToolSchema;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;
import tools.jackson.databind.JsonNode;

import java.util.List;
import java.util.Map;

@Component
@RequiredArgsConstructor
public class ContextTool implements McpTool {

    private final ContextService context;
    private final ContextRenderer renderer;

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

               A note marked `loaded="on request"` arrives as its title, its first line and an
               anchor such as `note:3f2a9c1e`, not its body. Its person judged most sessions do
               not need it. If the work does, call this tool with that anchor and the note comes
               back in full; if it does not, leave it.

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

               A task may also hand over commits. Each `<commit>` inside `<commits>` is a change
               the person chose in the console to travel with this context: its hash, its subject,
               the step it was logged against if any, and the diff it introduced. Read a diff as
               the change it records, not as the current state of the file: it is where to start
               looking, and the repository is what to trust. A task with no `<commits>` block has
               nothing chosen; `rekall_record_commit` logs a commit but does not choose it.

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
        return renderer.render(load(anchors));
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
}
