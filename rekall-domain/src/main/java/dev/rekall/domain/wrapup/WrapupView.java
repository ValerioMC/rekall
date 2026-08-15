package dev.rekall.domain.wrapup;

import dev.rekall.domain.Wrapup;
import dev.rekall.domain.WrapupAuthor;

import java.time.Instant;
import java.util.UUID;

/**
 * One wrapup, fully materialised.
 *
 * <p>Materialised for the same reason {@code ContextRecord} is: it is read inside a transaction
 * and rendered outside one, and handing the entity to the renderer instead would make every
 * field access a bet on whether the session is still open.
 *
 * @param anchor what you would type after {@code /rk} to load the task this describes
 */
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
