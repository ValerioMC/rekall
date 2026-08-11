package dev.rekall.engine.schema;

/**
 * A column as it actually exists in the database.
 *
 * <p>Read from {@code information_schema}, never inferred from what the engine believes it
 * created. If someone alters the schema by hand, the diff sees the real state.
 *
 * @param dataType {@code information_schema} type name, e.g. {@code character varying}
 * @param udtName underlying type name, needed to tell {@code text[]} from any other {@code ARRAY}
 */
public record PhysicalColumn(
        String name,
        String dataType,
        String udtName,
        Integer characterMaximumLength,
        Integer numericPrecision,
        Integer numericScale,
        boolean nullable,
        String defaultExpression) {

    /** Compact form used in plan warnings, e.g. {@code character varying(255)}. */
    public String describeType() {
        if (characterMaximumLength != null) {
            return dataType + "(" + characterMaximumLength + ")";
        }
        if ("numeric".equals(dataType) && numericPrecision != null) {
            return dataType + "(" + numericPrecision + "," + (numericScale == null ? 0 : numericScale) + ")";
        }
        if ("ARRAY".equals(dataType)) {
            return udtName.startsWith("_") ? udtName.substring(1) + "[]" : udtName;
        }
        return dataType;
    }
}
