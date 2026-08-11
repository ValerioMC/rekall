package dev.rekall.engine.plan;

/** Outcome of comparing a declared field type against the column that exists. */
public enum TypeComparison {

    /** The physical column already is what the meta-model declares. */
    MATCHES,

    /** The change can only add capacity, so no existing value can fail to fit. */
    SAFE_WIDENING,

    /**
     * Anything else. Narrowing a varchar, changing a date to a number, swapping an array for a
     * scalar: all of these need a {@code USING} cast whose failure mode is silent data loss,
     * so the planner refuses instead of guessing.
     */
    UNSAFE
}
