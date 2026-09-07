package dev.rekall.domain.step;

import dev.rekall.domain.TaskStep;
import dev.rekall.domain.TaskStepState;

import java.time.Instant;
import java.util.UUID;

/**
 * One step, fully materialised, for the same reason {@code WrapupView} is: read inside a
 * transaction and rendered outside one.
 *
 * <p>{@code done} rides alongside {@code state} rather than being dropped for it: it is
 * {@code state == DONE}, and it is what the navigator's progress count and the "finished since
 * the wrapup" check are already written against. {@code state} is the fuller answer the console
 * animates around.
 *
 * @param doneAt when a person accepted the work, or null short of that
 * @param runningAt when a session picked the step up, or null
 * @param claimedAt when a session claimed it as finished, or null
 */
public record TaskStepView(
        UUID id,
        UUID taskId,
        String title,
        String bodyMarkdown,
        TaskStepState state,
        boolean done,
        Instant runningAt,
        Instant claimedAt,
        Instant doneAt,
        int position,
        Instant createdAt,
        Instant updatedAt) {

    public static TaskStepView of(TaskStep step) {
        return new TaskStepView(
                step.getId(),
                step.getTask().getId(),
                step.getTitle(),
                step.getBodyMarkdown(),
                step.getState(),
                step.isDone(),
                step.getRunningAt(),
                step.getClaimedAt(),
                step.getDoneAt(),
                step.getPosition(),
                step.getCreatedAt(),
                step.getUpdatedAt());
    }

    /** The moment the work was finished, {@code claimedAt} first, for the wrapup-behind check. */
    public Instant completedAt() {
        return claimedAt != null ? claimedAt : doneAt;
    }
}
