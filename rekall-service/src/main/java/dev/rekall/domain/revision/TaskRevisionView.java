package dev.rekall.domain.revision;

import dev.rekall.domain.RevisionKind;
import dev.rekall.domain.TaskRevision;
import dev.rekall.domain.WrapupAuthor;

import java.time.Instant;
import java.util.UUID;

public record TaskRevisionView(
        UUID id,
        UUID taskId,
        RevisionKind kind,
        String bodyMarkdown,
        WrapupAuthor writtenBy,
        Instant writtenAt,
        Instant replacedAt) {

    public static TaskRevisionView of(TaskRevision revision) {
        return new TaskRevisionView(
                revision.getId(),
                revision.getTask().getId(),
                revision.getKind(),
                revision.getBodyMarkdown(),
                revision.getWrittenBy(),
                revision.getWrittenAt(),
                revision.getCreatedAt());
    }
}
