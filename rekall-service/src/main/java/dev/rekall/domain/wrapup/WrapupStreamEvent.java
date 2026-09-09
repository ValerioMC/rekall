package dev.rekall.domain.wrapup;

import java.util.UUID;

/**
 * Emitted whenever a task's wrapup is written or deleted, so a console watching
 * the step SSE feed sees the new text the moment it commits rather than on the
 * next reload. On a delete {@code wrapup} is null and {@code deleted} is true.
 */
public record WrapupStreamEvent(UUID taskId, WrapupView wrapup, boolean deleted) {

    public static WrapupStreamEvent written(WrapupView wrapup) {
        return new WrapupStreamEvent(wrapup.taskId(), wrapup, false);
    }

    public static WrapupStreamEvent deleted(UUID taskId) {
        return new WrapupStreamEvent(taskId, null, true);
    }
}
