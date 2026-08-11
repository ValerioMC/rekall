package dev.rekall.meta.domain;

/**
 * Cardinality of a relation between two entities.
 *
 * <p>{@code ONE_TO_MANY} is deliberately absent: it is the inverse view of a
 * {@link #MANY_TO_ONE} and is derived when rendering the schema. Storing it would allow two
 * rows describing the same foreign key to disagree.
 */
public enum RelationKind {

    /** Foreign key column on the source entity pointing at the target entity. */
    MANY_TO_ONE,

    /** Generated join table holding a pair of foreign keys. */
    MANY_TO_MANY
}
