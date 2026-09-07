package dev.rekall.domain.wrapup;

import dev.rekall.domain.Wrapup;
import dev.rekall.domain.WrapupAuthor;

import java.time.Instant;
import java.util.UUID;

public record WrapupView(
        UUID id,
        UUID taskId,
        String taskLabel,
        String taskTitle,
        String projectLabel,
        String anchor,
        String bodyMarkdown,
        WrapupAuthor writtenBy,
        Instant createdAt,
        Instant updatedAt) {

    public static WrapupView of(Wrapup wrapup) {
        return new WrapupView(
                wrapup.getId(),
                wrapup.getTask().getId(),
                wrapup.getTask().getLabel(),
                wrapup.getTask().getTitle(),
                wrapup.getTask().getProject().getLabel(),
                wrapup.anchor(),
                wrapup.getBodyMarkdown(),
                wrapup.getWrittenBy(),
                wrapup.getCreatedAt(),
                wrapup.getUpdatedAt());
    }
}
