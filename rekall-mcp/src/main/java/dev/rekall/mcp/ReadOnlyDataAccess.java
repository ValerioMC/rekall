package dev.rekall.mcp;

import dev.rekall.engine.data.DynamicRecordRepository;
import dev.rekall.engine.data.JooqDynamicRecordRepository;
import dev.rekall.engine.data.QueryFilter;
import dev.rekall.engine.data.RecordView;
import dev.rekall.engine.schema.SchemaRegistry;
import dev.rekall.engine.type.PostgresTypeMapper;
import dev.rekall.meta.domain.MetaTable;
import org.jooq.DSLContext;
import org.jooq.Record;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * Everything the MCP tools are allowed to do.
 *
 * <p>The record repository is constructed here rather than injected: the application's
 * instance is a Spring bean bound to the read-write datasource and its transaction manager,
 * and reusing it would quietly undo the separation this class exists to enforce. Constructing
 * it directly also keeps it out of the transactional proxy, which is correct because every
 * call below is a single statement.
 */
@Component
public class ReadOnlyDataAccess {

    private final DynamicRecordRepository records;
    private final DSLContext readerDsl;
    private final SchemaRegistry registry;

    public ReadOnlyDataAccess(
            @Qualifier("mcpReaderDslContext") DSLContext readerDsl,
            PostgresTypeMapper typeMapper,
            SchemaRegistry registry) {
        this.readerDsl = readerDsl;
        this.registry = registry;
        this.records = new JooqDynamicRecordRepository(readerDsl, typeMapper, registry);
    }

    public SchemaRegistry schema() {
        return registry;
    }

    public Optional<MetaTable> resolveEntity(String name) {
        return registry.resolveEntity(name);
    }

    public List<RecordView> query(MetaTable entity, QueryFilter filter, int resolveDepth) {
        return records.query(entity, filter, resolveDepth);
    }

    public long count(MetaTable entity, QueryFilter filter) {
        return records.count(entity, filter);
    }

    public List<RecordView> findByLabel(MetaTable entity, String label, int resolveDepth) {
        return records.findByLabel(entity, label, resolveDepth);
    }

    public Optional<RecordView> findById(MetaTable entity, UUID id, int resolveDepth) {
        return records.findById(entity, id, resolveDepth);
    }

    /** Documents attached to a record, read through the reader identity like everything else. */
    public List<DocumentSummary> documentsFor(String entityName, UUID recordId) {
        return readerDsl
                .fetch(
                        """
                        SELECT id, title, kind, body_markdown
                        FROM rekall_meta.document
                        WHERE entity_name = ? AND record_id = ?
                        ORDER BY position
                        """,
                        entityName,
                        recordId)
                .map(this::toDocumentSummary);
    }

    private DocumentSummary toDocumentSummary(Record row) {
        return new DocumentSummary(
                row.get("id", UUID.class),
                row.get("title", String.class),
                row.get("kind", String.class),
                row.get("body_markdown", String.class));
    }

    public record DocumentSummary(UUID id, String title, String kind, String bodyMarkdown) {
    }
}
