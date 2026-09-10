package dev.rekall.domain.review;

import java.util.UUID;

/** Emitted whenever a stepless task's review line moves, over the same SSE feed the steps use. */
public record TaskReviewEvent(UUID taskId, TaskReviewView review) {
}
