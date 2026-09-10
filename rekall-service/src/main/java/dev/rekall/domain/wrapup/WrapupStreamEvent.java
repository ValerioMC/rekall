package dev.rekall.domain.wrapup;

import java.util.UUID;

/** Emitted when a task's wrapup is written or deleted. On a delete {@code wrapup} is null. */
public record WrapupStreamEvent(UUID taskId, WrapupView wrapup, boolean deleted) {

    public static WrapupStreamEvent written(WrapupView wrapup) {
        return new WrapupStreamEvent(wrapup.taskId(), wrapup, false);
    }

    public static WrapupStreamEvent deleted(UUID taskId) {
        return new WrapupStreamEvent(taskId, null, true);
    }
}
