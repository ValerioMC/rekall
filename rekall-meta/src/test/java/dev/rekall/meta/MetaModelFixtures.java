package dev.rekall.meta;

import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaFieldType;
import dev.rekall.meta.domain.MetaTable;

/**
 * Builders for the meta-model shapes the tests keep needing.
 *
 * <p>They mirror the entities the application is actually for, so a failure reads like a
 * problem with projects and tasks rather than with {@code table_a} and {@code col_1}.
 */
public final class MetaModelFixtures {

    private MetaModelFixtures() {
    }

    public static MetaTable project() {
        return new MetaTable("project", "Project", "Projects", "Tutti i progetti su cui si sta lavorando");
    }

    public static MetaTable task() {
        return new MetaTable("task", "Task", "Tasks", "Attivita e issue legate a un progetto");
    }

    public static MetaTable environment() {
        return new MetaTable(
                "environment", "Environment", "Environments", "Configurazioni di cluster, namespace e database");
    }

    public static MetaField text(String columnName, String label) {
        return new MetaField(columnName, label, "Campo di test: " + label, MetaFieldType.TEXT);
    }

    public static MetaField text(String columnName, String label, int length) {
        MetaField field = text(columnName, label);
        field.setLength(length);
        return field;
    }

    public static MetaField longText(String columnName, String label) {
        return new MetaField(columnName, label, "Campo di test: " + label, MetaFieldType.LONG_TEXT);
    }

    public static MetaField integer(String columnName, String label) {
        return new MetaField(columnName, label, "Campo di test: " + label, MetaFieldType.INTEGER);
    }

    public static MetaField tags(String columnName, String label) {
        return new MetaField(columnName, label, "Campo di test: " + label, MetaFieldType.TAGS);
    }

    public static MetaField reference(String columnName, String label) {
        return new MetaField(columnName, label, "Campo di test: " + label, MetaFieldType.REFERENCE);
    }

    public static MetaField timestamp(String columnName, String label) {
        return new MetaField(columnName, label, "Campo di test: " + label, MetaFieldType.TIMESTAMP);
    }

    public static MetaField decimal(String columnName, String label, int precision, int scale) {
        MetaField field = new MetaField(columnName, label, "Campo di test: " + label, MetaFieldType.DECIMAL);
        field.setPrecision(precision);
        field.setScale(scale);
        return field;
    }

    public static MetaField enumeration(String columnName, String label, String... values) {
        MetaField field = new MetaField(columnName, label, "Campo di test: " + label, MetaFieldType.ENUM);
        field.setEnumValues(values);
        field.setLength(50);
        return field;
    }

    /** Marks a field as required, returning it so it can be used inline in a builder chain. */
    public static MetaField required(MetaField field) {
        field.setNullable(false);
        return field;
    }

    public static MetaField withDefault(MetaField field, String defaultValue) {
        field.setDefaultValue(defaultValue);
        return field;
    }
}
