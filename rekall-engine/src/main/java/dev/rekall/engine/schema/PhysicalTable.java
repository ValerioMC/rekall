package dev.rekall.engine.schema;

import java.util.Map;
import java.util.Optional;

/** A table as it actually exists in {@code rekall_data}. */
public record PhysicalTable(String name, Map<String, PhysicalColumn> columns, Map<String, PhysicalForeignKey> foreignKeys) {

    public Optional<PhysicalColumn> column(String columnName) {
        return Optional.ofNullable(columns.get(columnName));
    }

    public boolean hasColumn(String columnName) {
        return columns.containsKey(columnName);
    }
}
