package dev.rekall.meta.domain;

/** Outcome of a single statement inside an applied DDL plan. */
public enum DdlStatus {

    /** Statement executed and the surrounding transaction committed. */
    APPLIED,

    /** Statement raised an error. The whole plan was rolled back. */
    FAILED,

    /**
     * Statement belonged to a plan that was rolled back because a later statement failed.
     * Recorded so the log explains the full attempt, not only the statement that broke.
     */
    ROLLED_BACK
}
