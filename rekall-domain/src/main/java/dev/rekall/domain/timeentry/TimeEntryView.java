package dev.rekall.domain.timeentry;

import dev.rekall.domain.TimeEntry;

import java.time.Instant;
import java.util.UUID;

/**
 * One session, fully materialised, for the same reason {@code WrapupView} is: read inside a
 * transaction, rendered outside one.
 *
 * @param stoppedAt null while the session is still open
 * @param anchor what you would type after {@code /rk} to load the task this session was on
 */
public record TimeEntryView(
        UUID id,
        UUID taskId,
        String taskLabel,
        String taskTitle,
        String projectLabel,
        String anchor,
        Instant startedAt,
        Instant stoppedAt,
        Instant createdAt,
        Instant updatedAt) {

    public static TimeEntryView of(TimeEntry entry) {
        return new TimeEntryView(
                entry.getId(),
                entry.getTask().getId(),
                entry.getTask().getLabel(),
                entry.getTask().getTitle(),
                entry.getTask().getProject().getLabel(),
                entry.anchor(),
                entry.getStartedAt(),
                entry.getStoppedAt(),
                entry.getCreatedAt(),
                entry.getUpdatedAt());
    }
}
