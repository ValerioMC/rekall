package dev.rekall.domain.timeentry;

import dev.rekall.domain.TimeEntry;

import java.time.Instant;
import java.util.UUID;

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
