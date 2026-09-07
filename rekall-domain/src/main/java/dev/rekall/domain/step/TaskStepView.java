package dev.rekall.domain.step;

import dev.rekall.domain.TaskStep;
import dev.rekall.domain.TaskStepState;

import java.time.Instant;
import java.util.UUID;

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

    public Instant completedAt() {
        return claimedAt != null ? claimedAt : doneAt;
    }
}
