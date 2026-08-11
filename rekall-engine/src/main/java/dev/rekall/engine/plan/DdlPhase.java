package dev.rekall.engine.plan;

/**
 * Execution order of a plan.
 *
 * <p>Every foreign key is deferred to {@link #ADD_CONSTRAINT}, after all tables and columns
 * exist. That single decision removes any dependency on creation order, including circular
 * references between two entities created in the same plan.
 */
public enum DdlPhase {

    /** New tables with their system columns and user columns, without any foreign key. */
    CREATE_TABLE,

    /** Triggers and indexes belonging to a table created in this plan. */
    TABLE_INFRASTRUCTURE,

    /** Column additions, alterations and removals on tables that already existed. */
    ALTER_COLUMN,

    /** Foreign keys, added once every table and column they mention is in place. */
    ADD_CONSTRAINT,

    /** Join tables backing many-to-many relations. */
    JOIN_TABLE,

    /** Constraint, table and document removals, last so nothing depends on them any more. */
    DROP
}
