package dev.rekall.domain.review;

import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStepState;

import java.time.Instant;
import java.util.UUID;

/**
 * The task-scoped review line as the console reads it. {@code reviewActive} is false once the task
 * has a checklist, and the rest then means nothing.
 */
public record TaskReviewView(
        UUID taskId,
        TaskStepState reviewState,
        boolean reviewActive,
        Instant claimedAt,
        Instant acceptedAt,
        String reviewNote) {

    public static TaskReviewView of(Task task) {
        return new TaskReviewView(
                task.getId(),
                task.getReviewState(),
                task.reviewActive(),
                task.getClaimedAt(),
                task.getAcceptedAt(),
                task.getReviewNote());
    }
}
