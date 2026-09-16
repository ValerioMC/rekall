package dev.rekall.domain.commit;

import dev.rekall.domain.CommitReference;

import java.time.Instant;
import java.util.UUID;

public record CommitReferenceView(
        UUID id,
        UUID taskId,
        UUID stepId,
        String stepTitle,
        String commitHash,
        String comment,
        boolean inContext,
        Instant createdAt) {

    static CommitReferenceView of(CommitReference reference) {
        return new CommitReferenceView(
                reference.getId(),
                reference.getTask().getId(),
                reference.getStep() == null ? null : reference.getStep().getId(),
                reference.getStep() == null ? null : reference.getStep().getTitle(),
                reference.getCommitHash(),
                reference.getComment(),
                reference.isInContext(),
                reference.getCreatedAt());
    }
}
