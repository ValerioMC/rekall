package dev.rekall.api.service;

import dev.rekall.engine.data.DynamicRecordRepository;
import dev.rekall.engine.data.QueryFilter;
import dev.rekall.engine.data.RecordView;
import dev.rekall.engine.schema.SchemaRegistry;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.repository.DocumentRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.Map;
import java.util.UUID;

/** CRUD on records of dynamically generated entities, for the UI's data browser. */
@Service
@RequiredArgsConstructor
public class DataService {

    private final SchemaRegistry registry;
    private final DynamicRecordRepository records;
    private final DocumentRepository documents;

    public MetaTable requireEntity(String name) {
        return registry.resolveEntity(name)
                .orElseThrow(() -> new NotFoundException("No entity named '%s'".formatted(name)));
    }

    @Transactional(readOnly = true)
    public List<RecordView> list(String entityName, QueryFilter filter, int resolveDepth) {
        return records.query(requireEntity(entityName), filter, resolveDepth);
    }

    @Transactional(readOnly = true)
    public long count(String entityName, QueryFilter filter) {
        return records.count(requireEntity(entityName), filter);
    }

    @Transactional(readOnly = true)
    public RecordView get(String entityName, UUID id) {
        MetaTable entity = requireEntity(entityName);
        return records.findById(entity, id, 1)
                .orElseThrow(() -> new NotFoundException("record of " + entity.getPhysicalName(), id));
    }

    @Transactional
    public RecordView create(String entityName, Map<String, Object> values) {
        MetaTable entity = requireEntity(entityName);
        UUID id = records.insert(entity, values);
        return records.findById(entity, id, 1).orElseThrow(() -> new NotFoundException("record", id));
    }

    @Transactional
    public RecordView update(String entityName, UUID id, Map<String, Object> values) {
        MetaTable entity = requireEntity(entityName);
        records.update(entity, id, values);
        return records.findById(entity, id, 1).orElseThrow(() -> new NotFoundException("record", id));
    }

    /**
     * Deleting a record takes its documents with it, for the same reason dropping an entity
     * does: the reference from {@code document} into {@code rekall_data} is soft, so no
     * cascade will do it.
     */
    @Transactional
    public void delete(String entityName, UUID id) {
        MetaTable entity = requireEntity(entityName);
        if (!records.delete(entity, id)) {
            throw new NotFoundException("record of " + entity.getPhysicalName(), id);
        }
        documents.deleteByEntityNameAndRecordId(entity.getPhysicalName(), id);
    }
}
