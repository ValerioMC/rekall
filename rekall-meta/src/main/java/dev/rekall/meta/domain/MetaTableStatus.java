package dev.rekall.meta.domain;

/**
 * Lifecycle of an entity definition relative to the physical schema.
 *
 * <p>The status is advisory for the UI. The authority on what exists physically is always the
 * live diff against {@code information_schema}, never this field.
 */
public enum MetaTableStatus {

    /** Defined in the meta-model, no physical table yet. */
    DRAFT,

    /** Physical table exists and matches the definition as of the last apply. */
    APPLIED,

    /** Physical table exists but the definition has changed since. */
    MODIFIED
}
