package dev.rekall.domain.commit;

import java.util.UUID;

/** Emitted when a commit is logged against a task or step, whichever path logged it. */
public record CommitReferenceStreamEvent(UUID taskId, CommitReferenceView reference) {

    public static CommitReferenceStreamEvent logged(CommitReferenceView reference) {
        return new CommitReferenceStreamEvent(reference.taskId(), reference);
    }
}
