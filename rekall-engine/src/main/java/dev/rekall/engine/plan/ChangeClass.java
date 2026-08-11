package dev.rekall.engine.plan;

/**
 * How much trust a single schema change deserves.
 *
 * <p>The classification is what turns Execute from "run this DDL and hope" into a reviewable
 * plan. A plan containing anything {@link #BLOCKED} cannot be applied at all, and one with
 * unresolved {@link #NEEDS_INPUT} changes cannot be applied until the user answers.
 */
public enum ChangeClass {

    /** Cannot lose data and cannot fail on existing rows. Applied without asking. */
    SAFE,

    /** Applicable, but only once the user supplies a default or confirms a deletion. */
    NEEDS_INPUT,

    /** Refused. The user has to reach the goal a different way. */
    BLOCKED
}
