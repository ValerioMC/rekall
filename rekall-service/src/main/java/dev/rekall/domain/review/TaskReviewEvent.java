package dev.rekall.domain.review;

import java.util.UUID;

/**
 * Emitted whenever a stepless task's review line moves, so the console can update
 * the description lifecycle live over the same SSE feed the steps use.
 */
public record TaskReviewEvent(UUID taskId, TaskReviewView review) {
}
