package dev.rekall.domain.step;

import dev.rekall.domain.TaskStep;

import java.time.Instant;
import java.util.UUID;

/**
 * One step, fully materialised, for the same reason {@code WrapupView} is: read inside a
 * transaction and rendered outside one.
 *
 * @param doneAt when it was ticked, or null while it is still open
 */
public record TaskStepView(
        UUID id,
        UUID taskId,
        String title,
        String bodyMarkdown,
        boolean done,
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
                step.isDone(),
                step.getDoneAt(),
                step.getPosition(),
                step.getCreatedAt(),
                step.getUpdatedAt());
    }
}
