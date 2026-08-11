package dev.rekall.engine.schema;

import java.util.Map;
import java.util.Optional;
import java.util.Set;

/** Snapshot of everything that currently exists in {@code rekall_data}. */
public record PhysicalSchema(Map<String, PhysicalTable> tables) {

    public Optional<PhysicalTable> table(String name) {
        return Optional.ofNullable(tables.get(name));
    }

    public boolean hasTable(String name) {
        return tables.containsKey(name);
    }

    public Set<String> tableNames() {
        return tables.keySet();
    }
}
