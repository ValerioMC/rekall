package dev.rekall.meta.domain;

/** Referential action applied when the target row of a relation is deleted. */
public enum OnDeleteAction {

    /** Refuse the delete while referencing rows exist. The default, and the reason for real tables. */
    RESTRICT,

    /** Delete the referencing rows too. */
    CASCADE,

    /** Null out the referencing column. Only valid on a nullable reference field. */
    SET_NULL;

    public String toSql() {
        return switch (this) {
            case RESTRICT -> "RESTRICT";
            case CASCADE -> "CASCADE";
            case SET_NULL -> "SET NULL";
        };
    }
}
