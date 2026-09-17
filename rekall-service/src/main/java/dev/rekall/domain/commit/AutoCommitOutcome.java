package dev.rekall.domain.commit;

/**
 * What an automatic commit came to, for the session that triggered it. It never fails the claim
 * that caused it: a repository that cannot be committed is reported here, as {@link Status#FAILED},
 * and the claim stands.
 */
public record AutoCommitOutcome(Status status, String detail, CommitReferenceView reference) {

    public enum Status {
        /** The project does not auto-commit, or this claim is not one that commits. */
        OFF,
        /** The project auto-commits, but there was nothing in the working tree to commit. */
        SKIPPED,
        /** A commit was made and logged against the step or task. */
        COMMITTED,
        /** The project auto-commits, but git refused; {@code detail} says why. */
        FAILED
    }

    static AutoCommitOutcome notApplicable() {
        return new AutoCommitOutcome(Status.OFF, null, null);
    }

    static AutoCommitOutcome skipped(String detail) {
        return new AutoCommitOutcome(Status.SKIPPED, detail, null);
    }

    static AutoCommitOutcome committed(CommitReferenceView reference) {
        return new AutoCommitOutcome(Status.COMMITTED, null, reference);
    }

    static AutoCommitOutcome failed(String detail) {
        return new AutoCommitOutcome(Status.FAILED, detail, null);
    }

    public boolean off() {
        return status == Status.OFF;
    }

    /** The line a tool appends to its report: nothing when the project does not auto-commit. */
    public String describe() {
        return switch (status) {
            case OFF -> "";
            case SKIPPED -> "Auto-commit: " + detail;
            case FAILED -> "Auto-commit failed: " + detail + " The claim stands; commit and log by hand.";
            case COMMITTED -> "Auto-committed `%s` — %s — and logged it against %s. Do not commit this work again."
                    .formatted(
                            reference.commitHash().substring(0, Math.min(7, reference.commitHash().length())),
                            reference.comment(),
                            reference.stepTitle() == null ? "the task" : "\"" + reference.stepTitle() + "\"");
        };
    }
}
