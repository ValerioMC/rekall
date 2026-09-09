package dev.rekall.domain.review;

import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStepState;

import java.time.Instant;
import java.util.UUID;

/**
 * The task-scoped review line as the console reads it: the state, the two
 * moments on it, the optional send-back note, and whether it means anything for
 * this task at all ({@code reviewActive} is false once the task has a checklist).
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
