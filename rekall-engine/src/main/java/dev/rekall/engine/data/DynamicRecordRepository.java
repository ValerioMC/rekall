package dev.rekall.engine.data;

import dev.rekall.meta.domain.MetaTable;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;

/**
 * Reads and writes rows of tables that do not exist at compile time.
 *
 * <p>Nothing here is generated: fields are built from the meta-model at call time, which is
 * what lets the schema change without rebuilding the application.
 */
public interface DynamicRecordRepository {

    /** Deepest reference chain that will be followed, so a cycle cannot expand without bound. */
    int MAX_RESOLVE_DEPTH = 2;

    UUID insert(MetaTable entity, Map<String, Object> values);

    void update(MetaTable entity, UUID id, Map<String, Object> values);

    /** @return true if a row was removed */
    boolean delete(MetaTable entity, UUID id);

    Optional<RecordView> findById(MetaTable entity, UUID id, int resolveDepth);

    /** Resolves a uuid string or a display-field value to a record. */
    List<RecordView> findByLabel(MetaTable entity, String label, int resolveDepth);

    List<RecordView> query(MetaTable entity, QueryFilter filter, int resolveDepth);

    long count(MetaTable entity, QueryFilter filter);
}
