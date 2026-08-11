package dev.rekall.engine.schema;

/** A foreign key constraint as it actually exists in the database. */
public record PhysicalForeignKey(
        String constraintName, String sourceTable, String sourceColumn, String targetTable, String onDelete) {
}
