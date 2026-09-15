package dev.rekall.domain.context;

import dev.rekall.domain.CommitReference;

/**
 * One logged commit a task hands to {@code rekall_context}: the hash to look it up by, the
 * subject it was committed with, the step it was logged against, and the diff it introduced so
 * the session can read the change without a checkout.
 */
public record ContextCommitView(String commitHash, String comment, String stepTitle, String diff) {

    public static ContextCommitView of(CommitReference reference) {
        return new ContextCommitView(
                reference.getCommitHash(),
                reference.getComment(),
                reference.getStep() == null ? null : reference.getStep().getTitle(),
                reference.getDiff());
    }
}
