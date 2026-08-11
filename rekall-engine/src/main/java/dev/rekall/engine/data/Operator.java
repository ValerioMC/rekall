package dev.rekall.engine.data;

/** Comparison available in a {@link QueryFilter}. */
public enum Operator {
    EQ,
    NEQ,
    GT,
    GTE,
    LT,
    LTE,
    /** Case-sensitive pattern match, {@code %} as the wildcard. */
    LIKE,
    /** Case-insensitive pattern match. */
    ILIKE,
    IN,
    IS_NULL,
    IS_NOT_NULL,
    /** Array containment, for {@code TAGS} fields only. */
    CONTAINS;

    public boolean needsValue() {
        return this != IS_NULL && this != IS_NOT_NULL;
    }
}
