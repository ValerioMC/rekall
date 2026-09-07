package dev.rekall.domain.step;

import java.util.List;
import java.util.UUID;

/**
 * One task's checklist has changed, and the console holding it in memory needs to know.
 *
 * <p>Published by {@code TaskStepService} after every write, from the console or from MCP alike,
 * and fanned out to the open windows as Server-Sent Events by {@code StepEventStream} in
 * {@code rekall-api}. It is the read side of the loop the live-step feature is built on: a
 * session moves a step to {@code RUNNING} over MCP, and the animation in the console reacts
 * without anyone reloading the page.
 *
 * <p>It carries the whole of the task's checklist rather than the one step that moved. The list
 * is a handful of rows, a move renumbers all of them anyway, and "replace what you hold for this
 * task with this" is a rule that cannot leave the client half-updated.
 *
 * @param taskId the task whose checklist this is
 * @param steps every step on that task now, in order. Empty when the last one was removed
 */
public record StepStreamEvent(UUID taskId, List<TaskStepView> steps) {

    public StepStreamEvent {
        steps = steps == null ? List.of() : List.copyOf(steps);
    }
}
