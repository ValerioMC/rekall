package dev.rekall.engine.plan;

import dev.rekall.engine.RekallSchemas;
import dev.rekall.engine.type.PostgresTypeMapper;
import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaFieldType;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.OnDeleteAction;
import lombok.RequiredArgsConstructor;
import org.jooq.Constraint;
import org.jooq.DSLContext;
import org.jooq.DataType;
import org.jooq.Field;
import org.jooq.Name;
import org.jooq.Table;
import org.jooq.impl.DSL;
import org.jooq.impl.SQLDataType;
import org.springframework.stereotype.Component;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;

/**
 * Turns meta-model concepts into rendered Postgres DDL.
 *
 * <p>Every identifier passes through {@link DSL#name}, and every value through
 * {@link DSL#inline} after being coerced by the type mapper. No fragment of user input is ever
 * concatenated into a statement, which is the second of the two layers protecting generated
 * DDL. The first is {@code IdentifierValidator}, applied before anything is persisted.
 */
@Component
@RequiredArgsConstructor
public class DdlRenderer {

    private final DSLContext dsl;
    private final PostgresTypeMapper typeMapper;

    // ---------------------------------------------------------------- tables

    /** New table with its system columns and user columns. Foreign keys are added separately. */
    public String createTable(MetaTable meta) {
        List<Field<?>> columns = new ArrayList<>();
        columns.add(DSL.field(DSL.name("id"), SQLDataType.UUID.nullable(false)
                .defaultValue(DSL.field("gen_random_uuid()", SQLDataType.UUID))));
        columns.add(systemTimestamp("created_at"));
        columns.add(systemTimestamp("updated_at"));

        for (MetaField field : meta.getFields()) {
            columns.add(DSL.field(DSL.name(field.getColumnName()), columnType(field)));
        }

        List<Constraint> constraints = new ArrayList<>();
        constraints.add(DSL.constraint(DSL.name("pk_" + meta.getPhysicalName())).primaryKey(DSL.name("id")));
        meta.getFields().stream()
                .filter(field -> field.getType() == MetaFieldType.ENUM)
                .map(field -> enumCheck(meta, field))
                .forEach(constraints::add);

        return dsl.renderInlined(dsl.createTable(table(meta.getPhysicalName()))
                .columns(columns)
                .constraints(constraints));
    }

    public String dropTable(String physicalName) {
        return dsl.renderInlined(dsl.dropTable(table(physicalName)));
    }

    /**
     * Keeps {@code updated_at} honest without the application having to remember. The function
     * itself lives in {@code rekall_meta} and is created by Liquibase.
     */
    public String createUpdatedAtTrigger(MetaTable meta) {
        return dsl.renderInlined(dsl.query(
                "create trigger {0} before update on {1} for each row execute function "
                        + RekallSchemas.SET_UPDATED_AT_FUNCTION,
                DSL.name(triggerName(meta.getPhysicalName())),
                table(meta.getPhysicalName())));
    }

    /** GIN index backing containment queries on a {@code TAGS} column. */
    public String createTagsIndex(MetaTable meta, MetaField field) {
        return dsl.renderInlined(dsl.query(
                "create index {0} on {1} using gin ({2})",
                DSL.name(tagsIndexName(meta.getPhysicalName(), field.getColumnName())),
                table(meta.getPhysicalName()),
                DSL.field(DSL.name(field.getColumnName()))));
    }

    // --------------------------------------------------------------- columns

    public String addColumn(MetaTable meta, MetaField field) {
        return dsl.renderInlined(dsl.alterTable(table(meta.getPhysicalName()))
                .addColumn(DSL.field(DSL.name(field.getColumnName()), columnType(field))));
    }

    /**
     * First step of the three-step backfill: the column arrives nullable so that existing rows
     * do not violate it, and is tightened only after {@link #backfillColumn} has run.
     */
    public String addColumnNullable(MetaTable meta, MetaField field) {
        DataType<?> nullableType = columnType(field).nullable(true);
        return dsl.renderInlined(dsl.alterTable(table(meta.getPhysicalName()))
                .addColumn(DSL.field(DSL.name(field.getColumnName()), nullableType)));
    }

    @SuppressWarnings("unchecked")
    public String backfillColumn(MetaTable meta, MetaField field, String rawValue) {
        Object value = typeMapper.toJavaValue(field, rawValue);
        DataType<Object> type = (DataType<Object>) typeMapper.baseType(field);
        Field<Object> column = DSL.field(DSL.name(field.getColumnName()), type);

        // The map form of set(): with T inferred as Object the two-argument overloads
        // set(Field<T>, T) and set(Field<T>, Field<T>) are both applicable and neither is more
        // specific, so the typed form cannot be used for a column whose type is only known at
        // runtime.
        Field<Object> literal = DSL.inline(value, type);
        return dsl.renderInlined(dsl.update(table(meta.getPhysicalName()))
                .set(Map.of(column, literal))
                .where(column.isNull()));
    }

    public String setNotNull(MetaTable meta, MetaField field) {
        return dsl.renderInlined(dsl.alterTable(table(meta.getPhysicalName()))
                .alterColumn(DSL.name(field.getColumnName()))
                .setNotNull());
    }

    public String dropNotNull(MetaTable meta, MetaField field) {
        return dsl.renderInlined(dsl.alterTable(table(meta.getPhysicalName()))
                .alterColumn(DSL.name(field.getColumnName()))
                .dropNotNull());
    }

    public String alterColumnType(MetaTable meta, MetaField field) {
        return dsl.renderInlined(dsl.alterTable(table(meta.getPhysicalName()))
                .alterColumn(DSL.name(field.getColumnName()))
                .set(typeMapper.baseType(field)));
    }

    public String dropColumn(String physicalTableName, String columnName) {
        return dsl.renderInlined(dsl.alterTable(table(physicalTableName)).dropColumn(DSL.name(columnName)));
    }

    // ----------------------------------------------------------- constraints

    public String addForeignKey(MetaRelation relation, MetaField sourceField) {
        // references(Name, Name) rather than the typed Table/Field overload: the identifiers
        // are dynamic, so there is no compile-time type to infer and the name-based form says
        // exactly that instead of fighting generics with casts.
        var references = DSL.constraint(DSL.name(relation.constraintName()))
                .foreignKey(DSL.name(sourceField.getColumnName()))
                .references(tableName(relation.getTargetTable().getPhysicalName()), DSL.name("id"));

        Constraint constraint = switch (relation.getOnDelete()) {
            case RESTRICT -> references.onDeleteRestrict();
            case CASCADE -> references.onDeleteCascade();
            case SET_NULL -> references.onDeleteSetNull();
        };

        return dsl.renderInlined(
                dsl.alterTable(table(relation.getSourceTable().getPhysicalName())).add(constraint));
    }

    public String dropConstraint(String physicalTableName, String constraintName) {
        return dsl.renderInlined(
                dsl.alterTable(table(physicalTableName)).dropConstraint(DSL.name(constraintName)));
    }

    public String createJoinTable(MetaRelation relation) {
        String sourceColumn = joinColumn(relation, true);
        String targetColumn = joinColumn(relation, false);

        Table<?> joinTable = table(relation.getJoinTableName());
        Field<UUID> source = DSL.field(DSL.name(sourceColumn), SQLDataType.UUID.nullable(false));
        Field<UUID> target = DSL.field(DSL.name(targetColumn), SQLDataType.UUID.nullable(false));

        // Both sides cascade: a join row has no meaning once either end is gone, so RESTRICT
        // here would only ever be an obstacle to deleting a legitimately unwanted record.
        Constraint primaryKey = DSL.constraint(DSL.name("pk_" + relation.getJoinTableName()))
                .primaryKey(DSL.name(sourceColumn), DSL.name(targetColumn));
        Constraint sourceForeignKey = DSL.constraint(DSL.name("fk_" + relation.getJoinTableName() + "_source"))
                .foreignKey(DSL.name(sourceColumn))
                .references(tableName(relation.getSourceTable().getPhysicalName()), DSL.name("id"))
                .onDeleteCascade();
        Constraint targetForeignKey = DSL.constraint(DSL.name("fk_" + relation.getJoinTableName() + "_target"))
                .foreignKey(DSL.name(targetColumn))
                .references(tableName(relation.getTargetTable().getPhysicalName()), DSL.name("id"))
                .onDeleteCascade();

        return dsl.renderInlined(dsl.createTable(joinTable)
                .columns(source, target)
                .constraints(primaryKey, sourceForeignKey, targetForeignKey));
    }

    /**
     * Column naming for a join table. A self-referencing many-to-many would produce the same
     * name on both sides, so those fall back to {@code source_id} and {@code target_id}.
     */
    public static String joinColumn(MetaRelation relation, boolean source) {
        String sourceName = relation.getSourceTable().getPhysicalName();
        String targetName = relation.getTargetTable().getPhysicalName();
        if (sourceName.equals(targetName)) {
            return source ? "source_id" : "target_id";
        }
        return (source ? sourceName : targetName) + "_id";
    }

    public static String triggerName(String physicalTableName) {
        return "trg_" + physicalTableName + "_updated_at";
    }

    public static String tagsIndexName(String physicalTableName, String columnName) {
        return "idx_" + physicalTableName + "_" + columnName + "_gin";
    }

    public static String enumCheckName(String physicalTableName, String columnName) {
        return "ck_" + physicalTableName + "_" + columnName;
    }

    // ----------------------------------------------------------------- internals

    private Table<?> table(String physicalName) {
        return DSL.table(tableName(physicalName));
    }

    private Name tableName(String physicalName) {
        return DSL.name(RekallSchemas.DATA, physicalName);
    }

    private Field<?> systemTimestamp(String name) {
        return DSL.field(
                DSL.name(name),
                SQLDataType.TIMESTAMPWITHTIMEZONE
                        .nullable(false)
                        .defaultValue(DSL.field("now()", SQLDataType.TIMESTAMPWITHTIMEZONE)));
    }

    @SuppressWarnings("unchecked")
    private DataType<?> columnType(MetaField field) {
        DataType<Object> type = (DataType<Object>) typeMapper.toDataType(field);
        if (field.getDefaultValue() == null || field.getDefaultValue().isBlank()) {
            return type;
        }
        Object coerced = typeMapper.toJavaValue(field, field.getDefaultValue());
        return type.defaultValue(DSL.inline(coerced, type));
    }

    private Constraint enumCheck(MetaTable meta, MetaField field) {
        Field<String> column = DSL.field(DSL.name(field.getColumnName()), String.class);
        Name checkName = DSL.name(enumCheckName(meta.getPhysicalName(), field.getColumnName()));
        // A null passes a CHECK, so this constrains the value without implying NOT NULL.
        return DSL.constraint(checkName).check(column.in(field.getEnumValues()));
    }
}
