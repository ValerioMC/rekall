package dev.rekall.engine.data;

import dev.rekall.engine.RekallSchemas;
import dev.rekall.engine.schema.SchemaRegistry;
import dev.rekall.engine.type.PostgresTypeMapper;
import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaFieldType;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import lombok.RequiredArgsConstructor;
import org.jooq.Condition;
import org.jooq.DSLContext;
import org.jooq.DataType;
import org.jooq.Field;
import org.jooq.Record;
import org.jooq.Result;
import org.jooq.SortField;
import org.jooq.Table;
import org.jooq.impl.DSL;
import org.jooq.impl.SQLDataType;
import org.springframework.stereotype.Repository;
import org.springframework.transaction.annotation.Transactional;

import java.time.Instant;
import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.function.Function;
import java.util.stream.Collectors;

/**
 * Dynamic data access built from the meta-model at call time.
 *
 * <p>Fields are constructed with {@link DSL#field(org.jooq.Name, DataType)} rather than
 * generated constants, which is the whole reason the schema can change without a rebuild.
 * Values are always bound, never inlined.
 */
@Repository
@RequiredArgsConstructor
public class JooqDynamicRecordRepository implements DynamicRecordRepository {

    private static final Field<UUID> ID = DSL.field(DSL.name("id"), SQLDataType.UUID);
    private static final Field<OffsetDateTime> CREATED_AT =
            DSL.field(DSL.name("created_at"), SQLDataType.TIMESTAMPWITHTIMEZONE);
    private static final Field<OffsetDateTime> UPDATED_AT =
            DSL.field(DSL.name("updated_at"), SQLDataType.TIMESTAMPWITHTIMEZONE);

    private final DSLContext dsl;
    private final PostgresTypeMapper typeMapper;
    private final SchemaRegistry registry;

    // ----------------------------------------------------------------- writes

    @Override
    @Transactional
    public UUID insert(MetaTable entity, Map<String, Object> values) {
        Map<Field<?>, Object> row = coerceRow(entity, values);
        Record inserted = dsl.insertInto(table(entity))
                .set(row)
                .returningResult(ID)
                .fetchOne();
        if (inserted == null) {
            throw new IllegalStateException("Insert into %s returned no id".formatted(entity.getPhysicalName()));
        }
        return inserted.get(ID);
    }

    @Override
    @Transactional
    public void update(MetaTable entity, UUID id, Map<String, Object> values) {
        Map<Field<?>, Object> row = coerceRow(entity, values);
        if (row.isEmpty()) {
            return;
        }
        int updated = dsl.update(table(entity)).set(row).where(ID.eq(id)).execute();
        if (updated == 0) {
            throw new RecordNotFoundException(entity, id);
        }
    }

    @Override
    @Transactional
    public boolean delete(MetaTable entity, UUID id) {
        return dsl.deleteFrom(table(entity)).where(ID.eq(id)).execute() > 0;
    }

    // ------------------------------------------------------------------ reads

    @Override
    @Transactional(readOnly = true)
    public Optional<RecordView> findById(MetaTable entity, UUID id, int resolveDepth) {
        List<RecordView> found = query(entity, QueryFilter.where(QueryFilter.Condition.eq("id", id)), resolveDepth);
        return found.stream().findFirst();
    }

    @Override
    @Transactional(readOnly = true)
    public List<RecordView> findByLabel(MetaTable entity, String label, int resolveDepth) {
        Optional<UUID> asId = parseUuid(label);
        if (asId.isPresent()) {
            return findById(entity, asId.get(), resolveDepth).map(List::of).orElseGet(List::of);
        }
        Optional<MetaField> displayField = entity.displayField();
        if (displayField.isEmpty()) {
            return List.of();
        }
        return query(
                entity,
                QueryFilter.where(QueryFilter.Condition.eq(displayField.get().getColumnName(), label)),
                resolveDepth);
    }

    @Override
    @Transactional(readOnly = true)
    public List<RecordView> query(MetaTable entity, QueryFilter filter, int resolveDepth) {
        Result<Record> rows = dsl.select(selectFields(entity))
                .from(table(entity))
                .where(conditions(entity, filter))
                .orderBy(sortFields(entity, filter))
                .limit(filter.limit())
                .offset(filter.offset())
                .fetch();

        List<RecordView> views = rows.stream().map(row -> toView(entity, row)).toList();
        return resolveReferences(entity, views, resolveDepth);
    }

    @Override
    @Transactional(readOnly = true)
    public long count(MetaTable entity, QueryFilter filter) {
        return dsl.fetchCount(table(entity), conditions(entity, filter));
    }

    // ------------------------------------------------------ reference resolution

    /**
     * Replaces reference ids with the records they point at, one relation at a time and one
     * batch query per relation.
     *
     * <p>Batching rather than joining keeps the main query readable and, more importantly,
     * avoids the row multiplication a chain of joins would cause. The alternative, resolving
     * each row on its own, is the N+1 that would make loading a task list quietly expensive.
     */
    private List<RecordView> resolveReferences(MetaTable entity, List<RecordView> views, int depth) {
        if (depth <= 0 || views.isEmpty()) {
            return views;
        }
        int effectiveDepth = Math.min(depth, MAX_RESOLVE_DEPTH);
        List<RecordView> resolved = new ArrayList<>(views);

        for (MetaRelation relation : registry.outgoingRelations(entity.getPhysicalName())) {
            if (relation.getSourceFieldId() == null) {
                continue;
            }
            Optional<MetaField> sourceField = entity.getFields().stream()
                    .filter(field -> relation.getSourceFieldId().equals(field.getId()))
                    .findFirst();
            if (sourceField.isEmpty()) {
                continue;
            }
            String columnName = sourceField.get().getColumnName();

            Set<UUID> targetIds = resolved.stream()
                    .map(view -> view.value(columnName))
                    .filter(UUID.class::isInstance)
                    .map(UUID.class::cast)
                    .collect(Collectors.toCollection(LinkedHashSet::new));
            if (targetIds.isEmpty()) {
                continue;
            }

            MetaTable target = registry.entity(relation.getTargetTable().getPhysicalName()).orElse(null);
            if (target == null) {
                continue;
            }
            Map<UUID, RecordView> targetsById = query(
                            target,
                            new QueryFilter(
                                    List.of(new QueryFilter.Condition("id", Operator.IN, targetIds)),
                                    List.of(),
                                    QueryFilter.MAX_LIMIT,
                                    0),
                            effectiveDepth - 1)
                    .stream()
                    .collect(Collectors.toMap(RecordView::id, Function.identity(), (a, b) -> a));

            resolved.replaceAll(view -> {
                Object raw = view.value(columnName);
                RecordView targetView = raw instanceof UUID uuid ? targetsById.get(uuid) : null;
                return targetView == null ? view : view.withResolvedReference(columnName, targetView);
            });
        }
        return resolved;
    }

    // --------------------------------------------------------------- internals

    private Table<?> table(MetaTable entity) {
        return DSL.table(DSL.name(RekallSchemas.DATA, entity.getPhysicalName()));
    }

    private List<Field<?>> selectFields(MetaTable entity) {
        List<Field<?>> fields = new ArrayList<>();
        fields.add(ID);
        fields.add(CREATED_AT);
        fields.add(UPDATED_AT);
        entity.getFields().forEach(field -> fields.add(column(field)));
        return fields;
    }

    private Field<?> column(MetaField field) {
        return DSL.field(DSL.name(field.getColumnName()), typeMapper.baseType(field));
    }

    private RecordView toView(MetaTable entity, Record row) {
        Map<String, Object> values = new LinkedHashMap<>();
        for (MetaField field : entity.getFields()) {
            values.put(field.getColumnName(), row.get(DSL.name(field.getColumnName()).last()));
        }
        UUID id = row.get(ID);
        String label = entity.displayField()
                .map(field -> values.get(field.getColumnName()))
                .map(String::valueOf)
                .orElseGet(id::toString);

        return new RecordView(
                id,
                entity.getPhysicalName(),
                label,
                values,
                toInstant(row.get(CREATED_AT)),
                toInstant(row.get(UPDATED_AT)));
    }

    private Instant toInstant(OffsetDateTime value) {
        return value == null ? null : value.toInstant();
    }

    private Map<Field<?>, Object> coerceRow(MetaTable entity, Map<String, Object> values) {
        Map<Field<?>, Object> row = new LinkedHashMap<>();
        for (Map.Entry<String, Object> entry : values.entrySet()) {
            if (DdlSystemColumns.contains(entry.getKey())) {
                // Assigning id or the timestamps by hand would defeat the trigger and the
                // primary key default, so they are rejected rather than quietly honoured.
                continue;
            }
            MetaField field = entity.findField(entry.getKey())
                    .orElseThrow(() -> new UnknownFieldException(entity, entry.getKey()));
            row.put(column(field), typeMapper.toJavaValue(field, entry.getValue()));
        }
        return row;
    }

    private List<Condition> conditions(MetaTable entity, QueryFilter filter) {
        return filter.conditions().stream()
                .map(condition -> toJooqCondition(entity, condition))
                .toList();
    }

    @SuppressWarnings("unchecked")
    private Condition toJooqCondition(MetaTable entity, QueryFilter.Condition condition) {
        String name = condition.field();

        if ("id".equals(name)) {
            return comparison(ID, coerceAll(condition.value(), value -> toUuid(value)), condition.op());
        }
        if ("created_at".equals(name) || "updated_at".equals(name)) {
            Field<OffsetDateTime> field = "created_at".equals(name) ? CREATED_AT : UPDATED_AT;
            return comparison(field, coerceAll(condition.value(), this::toOffsetDateTime), condition.op());
        }

        MetaField metaField = entity.findField(name).orElseThrow(() -> new UnknownFieldException(entity, name));

        if (condition.op() == Operator.CONTAINS) {
            if (metaField.getType() != MetaFieldType.TAGS) {
                throw new IllegalArgumentException(
                        "CONTAINS applies to a TAGS field, and '%s' is %s".formatted(name, metaField.getType()));
            }
            // The bind has to carry the column's own type. Left to inference jOOQ binds a
            // String[] as varchar[], and Postgres refuses text[] @> varchar[].
            DataType<String[]> tagsType = (DataType<String[]>) typeMapper.baseType(metaField);
            Field<String[]> tags = (Field<String[]>) column(metaField);
            return DSL.condition(
                    "{0} @> {1}", tags, DSL.val(new String[] {String.valueOf(condition.value())}, tagsType));
        }

        Field<Object> field = (Field<Object>) column(metaField);
        Object value = coerceAll(condition.value(), raw -> typeMapper.toJavaValue(metaField, raw));
        return comparison(field, value, condition.op());
    }

    /** Applies the coercion element-wise for {@code IN}, and once otherwise. */
    private Object coerceAll(Object value, Function<Object, Object> coerce) {
        if (value == null) {
            return null;
        }
        if (value instanceof Collection<?> collection) {
            return collection.stream().map(coerce).toList();
        }
        return coerce.apply(value);
    }

    @SuppressWarnings("unchecked")
    private <T> Condition comparison(Field<T> field, Object value, Operator op) {
        if (op.needsValue() && value == null) {
            throw new IllegalArgumentException("Operator %s needs a value".formatted(op));
        }
        return switch (op) {
            case EQ -> field.eq((T) value);
            case NEQ -> field.ne((T) value);
            case GT -> field.gt((T) value);
            case GTE -> field.ge((T) value);
            case LT -> field.lt((T) value);
            case LTE -> field.le((T) value);
            case LIKE -> field.cast(String.class).like(String.valueOf(value));
            case ILIKE -> field.cast(String.class).likeIgnoreCase(String.valueOf(value));
            case IN -> field.in(value instanceof Collection<?> collection ? collection : List.of(value));
            case IS_NULL -> field.isNull();
            case IS_NOT_NULL -> field.isNotNull();
            case CONTAINS -> throw new IllegalArgumentException("CONTAINS is handled before this point");
        };
    }

    private List<SortField<?>> sortFields(MetaTable entity, QueryFilter filter) {
        if (filter.sort().isEmpty()) {
            // Newest first is the useful default for a memory: the task worked on last is the
            // one most likely to be asked about.
            return List.of(UPDATED_AT.desc());
        }
        return filter.sort().stream()
                .map(spec -> {
                    Field<?> field = switch (spec.field()) {
                        case "id" -> ID;
                        case "created_at" -> CREATED_AT;
                        case "updated_at" -> UPDATED_AT;
                        default -> column(entity.findField(spec.field())
                                .orElseThrow(() -> new UnknownFieldException(entity, spec.field())));
                    };
                    return spec.direction() == QueryFilter.SortSpec.Direction.ASC ? field.asc() : field.desc();
                })
                .collect(Collectors.toList());
    }

    private UUID toUuid(Object value) {
        return value instanceof UUID uuid ? uuid : UUID.fromString(String.valueOf(value));
    }

    private OffsetDateTime toOffsetDateTime(Object value) {
        return value instanceof OffsetDateTime time ? time : OffsetDateTime.parse(String.valueOf(value));
    }

    private Optional<UUID> parseUuid(String value) {
        try {
            return Optional.of(UUID.fromString(value));
        } catch (IllegalArgumentException e) {
            return Optional.empty();
        }
    }

    /** The columns Rekall generates on every table and manages itself. */
    private static final class DdlSystemColumns {
        private static final Set<String> NAMES = Set.of("id", "created_at", "updated_at");

        static boolean contains(String name) {
            return NAMES.contains(name);
        }
    }
}
