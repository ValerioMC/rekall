package dev.rekall.engine.data;

import java.time.Instant;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.UUID;

/**
 * One row of a generated table, as everything above the engine sees it.
 *
 * @param label the value of the entity's display field, or the id when none is configured.
 *     This is what makes a resolved reference readable: without it a task carries a bare uuid
 *     and the answer built from it is unusable
 * @param values column values by column name. A resolved reference holds a nested
 *     {@code RecordView} rather than the raw uuid
 */
public record RecordView(
        UUID id, String entityName, String label, Map<String, Object> values, Instant createdAt, Instant updatedAt) {

    public RecordView {
        values = values == null ? Map.of() : new LinkedHashMap<>(values);
    }

    /** Replaces a raw reference id with the record it points at. */
    public RecordView withResolvedReference(String columnName, RecordView target) {
        Map<String, Object> merged = new LinkedHashMap<>(values);
        merged.put(columnName, target);
        return new RecordView(id, entityName, label, merged, createdAt, updatedAt);
    }

    public Object value(String columnName) {
        return values.get(columnName);
    }
}
