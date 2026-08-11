package dev.rekall.engine.schema;

import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.RelationKind;
import dev.rekall.meta.repository.MetaRelationRepository;
import dev.rekall.meta.repository.MetaTableRepository;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import org.springframework.transaction.annotation.Transactional;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.Collectors;

/**
 * In-memory view of the applied meta-model.
 *
 * <p>Every dynamic query needs the field list and types of its entity, and reloading that from
 * {@code rekall_meta} on each call would put a handful of queries in front of every single
 * read. The cache is invalidated at exactly one point, the end of a successful DDL apply,
 * which is why it is a plain volatile snapshot rather than a Spring cache: there is no
 * eviction policy to reason about, only one explicit call.
 */
@Component
@RequiredArgsConstructor
@Slf4j
public class SchemaRegistry {

    private final MetaTableRepository metaTables;
    private final MetaRelationRepository metaRelations;

    /** Not final, so it stays out of the generated constructor. */
    private volatile Snapshot snapshot;

    /** Drops the cached view. The next read rebuilds it. */
    public void invalidate() {
        snapshot = null;
        log.debug("Schema registry invalidated");
    }

    public List<MetaTable> entities() {
        return List.copyOf(current().byPhysicalName().values());
    }

    public Optional<MetaTable> entity(String physicalName) {
        return Optional.ofNullable(current().byPhysicalName().get(physicalName));
    }

    /**
     * Resolves the name a user typed to an entity, trying the physical name, the label, the
     * plural label and the aliases in turn, case-insensitively.
     *
     * <p>This is what lets a question mention "progetti" and reach the {@code project} table
     * without any natural language processing: the mapping is data the user wrote when they
     * defined the entity.
     */
    public Optional<MetaTable> resolveEntity(String name) {
        if (name == null || name.isBlank()) {
            return Optional.empty();
        }
        return Optional.ofNullable(current().byAnyName().get(name.trim().toLowerCase(Locale.ROOT)));
    }

    /** Relations where this entity is the source, so the ones adding a column to it. */
    public List<MetaRelation> outgoingRelations(String physicalName) {
        return current().relations().stream()
                .filter(relation -> relation.getSourceTable().getPhysicalName().equals(physicalName))
                .toList();
    }

    public List<MetaRelation> incomingRelations(String physicalName) {
        return current().relations().stream()
                .filter(relation -> relation.getTargetTable().getPhysicalName().equals(physicalName))
                .toList();
    }

    public List<MetaRelation> relations() {
        return current().relations();
    }

    /** The many-to-one relation carried by a reference field, if there is one. */
    public Optional<MetaRelation> relationForField(UUID fieldId) {
        return current().relations().stream()
                .filter(relation -> relation.getKind() == RelationKind.MANY_TO_ONE)
                .filter(relation -> fieldId.equals(relation.getSourceFieldId()))
                .findFirst();
    }

    private Snapshot current() {
        Snapshot local = snapshot;
        if (local == null) {
            local = load();
            snapshot = local;
        }
        return local;
    }

    @Transactional(readOnly = true)
    protected Snapshot load() {
        List<MetaTable> tables = metaTables.findAllWithFields();
        List<MetaRelation> relations = metaRelations.findAllWithTables();

        Map<String, MetaTable> byPhysicalName = new LinkedHashMap<>();
        Map<String, MetaTable> byAnyName = new LinkedHashMap<>();

        for (MetaTable table : tables) {
            // Touch the field list while the session is open. The associations are eager, but
            // the collection itself is only guaranteed initialised by the entity graph.
            table.getFields().forEach(MetaField::getColumnName);

            byPhysicalName.put(table.getPhysicalName(), table);
            index(byAnyName, table.getPhysicalName(), table);
            index(byAnyName, table.getLabel(), table);
            index(byAnyName, table.getLabelPlural(), table);
            for (String alias : table.getAliases()) {
                index(byAnyName, alias, table);
            }
        }

        log.debug("Schema registry loaded: {} entities, {} relations", tables.size(), relations.size());
        return new Snapshot(byPhysicalName, byAnyName, relations);
    }

    /**
     * First definition wins. Two entities claiming the same alias is a modelling mistake, and
     * silently letting the later one shadow the earlier would make lookups depend on load
     * order, so the collision is logged instead.
     */
    private void index(Map<String, MetaTable> index, String name, MetaTable table) {
        if (name == null || name.isBlank()) {
            return;
        }
        String key = name.trim().toLowerCase(Locale.ROOT);
        MetaTable existing = index.putIfAbsent(key, table);
        if (existing != null && !existing.getPhysicalName().equals(table.getPhysicalName())) {
            log.warn(
                    "Name '{}' is claimed by both '{}' and '{}'. Lookups will resolve to '{}'.",
                    name,
                    existing.getPhysicalName(),
                    table.getPhysicalName(),
                    existing.getPhysicalName());
        }
    }

    /** Names of every entity, used by tests and by the MCP schema renderer. */
    public List<String> entityNames() {
        return current().byPhysicalName().keySet().stream().collect(Collectors.toList());
    }

    record Snapshot(
            Map<String, MetaTable> byPhysicalName, Map<String, MetaTable> byAnyName, List<MetaRelation> relations) {
    }
}
