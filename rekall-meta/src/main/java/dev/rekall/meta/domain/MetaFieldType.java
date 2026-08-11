package dev.rekall.meta.domain;

/**
 * Closed set of field types a user can pick from in the schema designer.
 *
 * <p>Raw SQL types never reach the DDL engine: every physical type is derived from one of
 * these constants by {@code PostgresTypeMapper}. Adding a type here is a deliberate act that
 * requires a mapping, a UI editor and a diff rule.
 */
public enum MetaFieldType {

    /** Short free text. Physical type {@code varchar(length)}, default length 255. */
    TEXT(true, false, false, false),

    /** Unbounded text. Physical type {@code text}. */
    LONG_TEXT(false, false, false, false),

    /** Unbounded text edited as markdown in the UI. Physical type {@code text}. */
    MARKDOWN(false, false, false, false),

    /** Whole number. Always {@code bigint}: this removes the integer to bigint migration entirely. */
    INTEGER(false, false, false, false),

    /** Exact decimal. Physical type {@code numeric(precision, scale)}. */
    DECIMAL(false, true, false, false),

    /** Physical type {@code boolean}. */
    BOOLEAN(false, false, false, false),

    /** Calendar date without time. Physical type {@code date}. */
    DATE(false, false, false, false),

    /** Instant with time zone. Physical type {@code timestamptz}. */
    TIMESTAMP(false, false, false, false),

    /**
     * Constrained text. Physical type {@code varchar} plus a generated
     * {@code CHECK (col IN (...))} derived from {@link MetaField#getEnumValues()}.
     * The check is generated, never user-authored, which keeps it inside the v1 scope.
     */
    ENUM(true, false, true, false),

    /** Set of labels. Physical type {@code text[]} with a GIN index. */
    TAGS(false, false, false, false),

    /** Foreign key to another entity. Physical type {@code uuid}, paired with a {@link MetaRelation}. */
    REFERENCE(false, false, false, true);

    private final boolean usesLength;
    private final boolean usesPrecisionScale;
    private final boolean usesEnumValues;
    private final boolean reference;

    MetaFieldType(boolean usesLength, boolean usesPrecisionScale, boolean usesEnumValues, boolean reference) {
        this.usesLength = usesLength;
        this.usesPrecisionScale = usesPrecisionScale;
        this.usesEnumValues = usesEnumValues;
        this.reference = reference;
    }

    public boolean usesLength() {
        return usesLength;
    }

    public boolean usesPrecisionScale() {
        return usesPrecisionScale;
    }

    public boolean usesEnumValues() {
        return usesEnumValues;
    }

    public boolean isReference() {
        return reference;
    }
}
