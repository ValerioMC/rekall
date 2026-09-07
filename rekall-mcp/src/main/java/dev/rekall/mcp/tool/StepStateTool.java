package dev.rekall.mcp.tool;

import dev.rekall.domain.TaskStepState;
import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.step.TaskStepService;
import dev.rekall.domain.step.TaskStepView;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.protocol.ToolSchema;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;
import tools.jackson.databind.JsonNode;

import java.util.List;
import java.util.Map;

/**
 * The write that moves one step of a task along its line.
 *
 * <p>A step is {@code OPEN}, then {@code RUNNING} while a session works on it, then
 * {@code CLAIMED} when that session says it is finished. This tool makes those three moves, and
 * the reopen back to {@code OPEN} when a session has to abandon a step. It cannot reach
 * {@code DONE}: that is a person in the console saying they reviewed the work. A session marking
 * its own work accepted is the one thing the split between {@code CLAIMED} and {@code DONE}
 * exists to prevent.
 *
 * <p>Driving the checklist is what this is for. A session opens on a task, moves the first open
 * step to {@code RUNNING}, does the work, writes the wrapup, claims the step, and moves to the
 * next one, all anchored through {@code /rk}. The console animates every move as it lands.
 */
@Component
@RequiredArgsConstructor
public class StepStateTool implements McpTool {

    private final TaskStepService steps;

    @Override
    public String name() {
        return "rekall_step";
    }

    @Override
    public boolean writes() {
        return true;
    }

    @Override
    public String description() {
        return """
               Move one step of a task along its line: `open` -> `running` -> `claimed`.

               Use it to drive a checklist. Mark the step you are about to work `running`, do \
               the work, write the wrapup, then mark it `claimed`. If a later step is still \
               open, mark that one `running` and go again.

               Anchor the task the way `rekall_context` does: `project:vega task:report-builder`. \
               It has to name exactly one task.

               `step` is the step's one-based position in the checklist, as `rekall_context` \
               lists them (`"3"`), or its exact title.

               `state` is one of:
               - `running`  a session is working on this step now
               - `claimed`  a session has finished it and it is waiting for the console to accept it
               - `open`     put a step back, for when a session abandons it

               There is no `done` here. A step is ticked done in the console, by the person who \
               reviewed the work. Ask for `claimed` when you are finished and leave the last \
               move to them.
               """;
    }

    @Override
    public Map<String, Object> inputSchema() {
        return ToolSchema.object()
                .requiredString(
                        "anchors",
                        "The task the step is on, e.g. `project:vega task:report-builder`. Labels, "
                                + "never titles, and it has to name exactly one task.")
                .requiredString(
                        "step",
                        "Which step: its one-based position in the checklist (`\"3\"`), or its exact title.")
                .requiredString(
                        "state",
                        "`running` when you start it, `claimed` when you are finished with it, or "
                                + "`open` to put it back. Never `done`: the console ticks that.")
                .build();
    }

    @Override
    public String execute(JsonNode arguments) {
        Arguments args = Arguments.of(arguments);
        AnchoredTask target = AnchoredTask.from(Anchor.parseAll(args.requiredString("anchors")));
        String stepRef = args.requiredString("step");
        String rawState = args.requiredString("state");
        TaskStepState requested = parseState(rawState);

        TaskStepView moved;
        try {
            moved = steps.transition(target.projectLabel(), target.taskLabel(), stepRef, requested);
        } catch (UnknownAnchorException e) {
            throw new ToolFailure(e.getMessage());
        } catch (AmbiguousAnchorException e) {
            throw new ToolFailure(e.getMessage() + ". Qualify it with `project:<label>`.");
        } catch (IllegalArgumentException e) {
            throw new ToolFailure(e.getMessage());
        }

        return report(target, moved, rawState);
    }

    // ------------------------------------------------------------------ output

    private String report(AnchoredTask target, TaskStepView moved, String rawState) {
        String anchor = anchorOf(target);
        List<TaskStepView> all = steps.findByTask(moved.taskId());
        int oneBased = moved.position() + 1;

        StringBuilder out = new StringBuilder("Step %d \"%s\" on `%s` is now `%s`.\n"
                .formatted(oneBased, moved.title(), anchor, moved.state().name().toLowerCase()));

        if (moved.state() == TaskStepState.CLAIMED) {
            if (meantDone(rawState)) {
                out.append("\nMarked `claimed`, not done: a session cannot tick the last box. ");
            } else {
                out.append("\nThe work is finished and waiting for the console to accept it. ");
            }
            out.append("Write the wrapup for this task now if you have not, folding in what this ")
                    .append("step built.\n");
        }

        all.stream()
                .filter(step -> step.state() == TaskStepState.OPEN)
                .findFirst()
                .ifPresentOrElse(
                        next -> out.append("\nNext open step: %d \"%s\". Start it with `/rk %s step:%d start`."
                                .formatted(next.position() + 1, next.title(), anchor, next.position() + 1)),
                        () -> {
                            if (allComplete(all)) {
                                out.append("\nEvery step is claimed or done. This task is finished bar the review.");
                            }
                        });

        return out.toString();
    }

    private boolean allComplete(List<TaskStepView> all) {
        return !all.isEmpty() && all.stream().allMatch(step -> step.state().complete());
    }

    private String anchorOf(AnchoredTask target) {
        return target.projectLabel() == null
                ? "task:" + target.taskLabel()
                : "project:%s task:%s".formatted(target.projectLabel(), target.taskLabel());
    }

    // ------------------------------------------------------------------ input

    /**
     * The state a word asks for.
     *
     * <p>Forgiving on the way in: a session may say `start`, `begin` or `in progress` for the
     * same move, and it may say `done` when it means "I am finished", which here is `claimed`.
     * {@link #meantDone} remembers that last case so the reply can be clear that the box is not
     * ticked.
     */
    private TaskStepState parseState(String raw) {
        String value = raw.strip().toLowerCase().replace('-', '_').replace(' ', '_');
        return switch (value) {
            case "running", "run", "start", "started", "starting", "begin", "in_progress", "progress" ->
                    TaskStepState.RUNNING;
            case "claimed", "claim", "done", "complete", "completed", "finish", "finished" ->
                    TaskStepState.CLAIMED;
            case "open", "reopen", "stop", "abandon", "todo", "back" -> TaskStepState.OPEN;
            default -> throw new ToolFailure(
                    "`state` is one of `running`, `claimed` or `open`. Got `%s`.".formatted(raw));
        };
    }

    private boolean meantDone(String raw) {
        String value = raw.strip().toLowerCase();
        return value.equals("done") || value.equals("complete") || value.equals("completed");
    }
}
