package dev.rekall.engine.plan;

import dev.rekall.engine.schema.PhysicalColumn;
import dev.rekall.engine.schema.PhysicalForeignKey;
import dev.rekall.engine.schema.PhysicalSchema;
import dev.rekall.engine.schema.PhysicalSchemaReader;
import dev.rekall.engine.schema.PhysicalTable;
import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaFieldType;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.RelationKind;
import dev.rekall.meta.repository.MetaRelationRepository;
import dev.rekall.meta.repository.MetaTableRepository;
import org.springframework.stereotype.Component;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.function.Function;
import java.util.stream.Collectors;

/**
 * Computes the difference between the meta-model and the live schema, and expresses it as an
 * ordered, classified plan.
 *
 * <p>Planning has no side effects. The same meta-model and the same {@link PlanOptions} always
 * produce the same plan, which is what makes the preview the user approves and the statements
 * that actually run the same thing.
 */
@Component
public class DdlPlanner {

    private final MetaTableRepository metaTables;
    private final MetaRelationRepository metaRelations;
    private final PhysicalSchemaReader schemaReader;
    private final PhysicalTypeComparator typeComparator;
    private final DdlRenderer renderer;

    public DdlPlanner(
            MetaTableRepository metaTables,
            MetaRelationRepository metaRelations,
            PhysicalSchemaReader schemaReader,
            PhysicalTypeComparator typeComparator,
            DdlRenderer renderer) {
        this.metaTables = metaTables;
        this.metaRelations = metaRelations;
        this.schemaReader = schemaReader;
        this.typeComparator = typeComparator;
        this.renderer = renderer;
    }

    @Transactional(readOnly = true)
    public DdlPlan plan(PlanOptions options) {
        List<MetaTable> declared = metaTables.findAllWithFields();
        List<MetaRelation> relations = metaRelations.findAllWithTables();
        PhysicalSchema physical = schemaReader.read();

        List<DdlStatement> statements = new ArrayList<>();
        Set<String> droppedEntities = new LinkedHashSet<>();
        Set<String> joinTableNames = relations.stream()
                .filter(relation -> relation.getKind() == RelationKind.MANY_TO_MANY)
                .map(MetaRelation::getJoinTableName)
                .collect(Collectors.toCollection(LinkedHashSet::new));

        planNewTables(declared, physical, statements);
        planExistingTables(declared, physical, options, statements);
        planForeignKeys(declared, relations, physical, statements);
        planJoinTables(relations, physical, statements);
        planDroppedTables(declared, physical, joinTableNames, options, statements, droppedEntities);

        return new DdlPlan(UUID.randomUUID(), statements, droppedEntities);
    }

    // ------------------------------------------------------------- new tables

    private void planNewTables(List<MetaTable> declared, PhysicalSchema physical, List<DdlStatement> statements) {
        for (MetaTable meta : declared) {
            if (physical.hasTable(meta.getPhysicalName())) {
                continue;
            }
            statements.add(DdlStatement.safe(
                    DdlPhase.CREATE_TABLE,
                    renderer.createTable(meta),
                    "Create table %s".formatted(meta.getPhysicalName())));
            statements.add(DdlStatement.safe(
                    DdlPhase.TABLE_INFRASTRUCTURE,
                    renderer.createUpdatedAtTrigger(meta),
                    "Maintain updated_at on %s".formatted(meta.getPhysicalName())));
            meta.getFields().stream()
                    .filter(field -> field.getType() == MetaFieldType.TAGS)
                    .forEach(field -> statements.add(DdlStatement.safe(
                            DdlPhase.TABLE_INFRASTRUCTURE,
                            renderer.createTagsIndex(meta, field),
                            "Index tags column %s.%s".formatted(meta.getPhysicalName(), field.getColumnName()))));
        }
    }

    // -------------------------------------------------------- existing tables

    private void planExistingTables(
            List<MetaTable> declared,
            PhysicalSchema physical,
            PlanOptions options,
            List<DdlStatement> statements) {

        for (MetaTable meta : declared) {
            Optional<PhysicalTable> existing = physical.table(meta.getPhysicalName());
            if (existing.isEmpty()) {
                continue;
            }
            PhysicalTable table = existing.get();
            long rowCount = schemaReader.countRows(meta.getPhysicalName());

            for (MetaField field : meta.getFields()) {
                Optional<PhysicalColumn> column = table.column(field.getColumnName());
                if (column.isEmpty()) {
                    planAddedColumn(meta, field, rowCount, options, statements);
                } else {
                    planChangedColumn(meta, field, column.get(), statements);
                }
            }

            planRemovedColumns(meta, table, options, statements);
        }
    }

    private void planAddedColumn(
            MetaTable meta,
            MetaField field,
            long rowCount,
            PlanOptions options,
            List<DdlStatement> statements) {

        String key = inputKey(meta.getPhysicalName(), field.getColumnName());
        boolean needsBackfill = !field.isNullable() && rowCount > 0 && !hasDefault(field);

        if (!needsBackfill) {
            statements.add(DdlStatement.safe(
                    DdlPhase.ALTER_COLUMN,
                    renderer.addColumn(meta, field),
                    "Add column %s.%s".formatted(meta.getPhysicalName(), field.getColumnName())));
            return;
        }

        Optional<String> backfill = options.backfillFor(key);
        if (backfill.isEmpty()) {
            statements.add(DdlStatement.needsInput(
                    DdlPhase.ALTER_COLUMN,
                    null,
                    "Add required column %s.%s".formatted(meta.getPhysicalName(), field.getColumnName()),
                    "%s already has %d row(s). A required column needs a value for them: supply a default."
                            .formatted(meta.getPhysicalName(), rowCount),
                    key));
            return;
        }

        // Three steps rather than one, so existing rows never momentarily violate the constraint.
        statements.add(DdlStatement.safe(
                DdlPhase.ALTER_COLUMN,
                renderer.addColumnNullable(meta, field),
                "Add column %s.%s as nullable".formatted(meta.getPhysicalName(), field.getColumnName())));
        statements.add(DdlStatement.safe(
                DdlPhase.ALTER_COLUMN,
                renderer.backfillColumn(meta, field, backfill.get()),
                "Backfill %s.%s with '%s' on %d existing row(s)"
                        .formatted(meta.getPhysicalName(), field.getColumnName(), backfill.get(), rowCount)));
        statements.add(DdlStatement.safe(
                DdlPhase.ALTER_COLUMN,
                renderer.setNotNull(meta, field),
                "Make %s.%s required".formatted(meta.getPhysicalName(), field.getColumnName())));
    }

    private void planChangedColumn(
            MetaTable meta, MetaField field, PhysicalColumn column, List<DdlStatement> statements) {

        TypeComparison comparison = typeComparator.compare(field, column);
        String qualified = "%s.%s".formatted(meta.getPhysicalName(), field.getColumnName());

        switch (comparison) {
            case SAFE_WIDENING -> statements.add(DdlStatement.safe(
                    DdlPhase.ALTER_COLUMN,
                    renderer.alterColumnType(meta, field),
                    "Widen %s from %s".formatted(qualified, column.describeType())));
            case UNSAFE -> statements.add(DdlStatement.blocked(
                    DdlPhase.ALTER_COLUMN,
                    "Change type of %s".formatted(qualified),
                    ("%s is %s and the definition asks for %s. This conversion can lose data, so it is refused. "
                            + "Delete the field and create a new one, then migrate the content deliberately.")
                            .formatted(qualified, column.describeType(), field.getType())));
            case MATCHES -> planNullabilityChange(meta, field, column, statements);
        }
    }

    private void planNullabilityChange(
            MetaTable meta, MetaField field, PhysicalColumn column, List<DdlStatement> statements) {

        if (field.isNullable() == column.nullable()) {
            return;
        }
        String qualified = "%s.%s".formatted(meta.getPhysicalName(), field.getColumnName());

        if (field.isNullable()) {
            statements.add(DdlStatement.safe(
                    DdlPhase.ALTER_COLUMN, renderer.dropNotNull(meta, field), "Make %s optional".formatted(qualified)));
            return;
        }

        // Tightening an existing column: the statement itself fails if any row holds a null,
        // and that failure rolls the whole plan back, so it is surfaced as a warning up front.
        statements.add(DdlStatement.needsInput(
                DdlPhase.ALTER_COLUMN,
                renderer.setNotNull(meta, field),
                "Make %s required".formatted(qualified),
                "Existing rows with an empty %s would make this fail and roll the whole plan back."
                        .formatted(field.getColumnName()),
                inputKey(meta.getPhysicalName(), field.getColumnName())));
    }

    private void planRemovedColumns(
            MetaTable meta, PhysicalTable table, PlanOptions options, List<DdlStatement> statements) {

        Set<String> declaredColumns =
                meta.getFields().stream().map(MetaField::getColumnName).collect(Collectors.toSet());

        for (PhysicalColumn column : table.columns().values()) {
            if (isSystemColumn(column.name()) || declaredColumns.contains(column.name())) {
                continue;
            }
            String key = inputKey(meta.getPhysicalName(), column.name());
            String description = "Drop column %s.%s".formatted(meta.getPhysicalName(), column.name());

            if (options.isDropConfirmed(key)) {
                statements.add(DdlStatement.safe(
                        DdlPhase.DROP, renderer.dropColumn(meta.getPhysicalName(), column.name()), description));
            } else {
                long rowCount = schemaReader.countRows(meta.getPhysicalName());
                statements.add(DdlStatement.needsInput(
                        DdlPhase.DROP,
                        null,
                        description,
                        "This destroys the values held by %d row(s). Confirm to proceed.".formatted(rowCount),
                        key));
            }
        }
    }

    // ----------------------------------------------------------- foreign keys

    private void planForeignKeys(
            List<MetaTable> declared,
            List<MetaRelation> relations,
            PhysicalSchema physical,
            List<DdlStatement> statements) {

        Map<UUID, MetaField> fieldsById = declared.stream()
                .flatMap(meta -> meta.getFields().stream())
                .collect(Collectors.toMap(MetaField::getId, Function.identity()));

        Set<String> declaredConstraints = new HashSet<>();

        for (MetaRelation relation : relations) {
            if (relation.getKind() != RelationKind.MANY_TO_ONE) {
                continue;
            }
            MetaField sourceField = fieldsById.get(relation.getSourceFieldId());
            if (sourceField == null) {
                statements.add(DdlStatement.blocked(
                        DdlPhase.ADD_CONSTRAINT,
                        "Relation %s".formatted(relation),
                        "The reference field backing this relation no longer exists. Delete the relation."));
                continue;
            }

            String constraintName = relation.constraintName();
            declaredConstraints.add(constraintName);

            boolean alreadyPresent = physical
                    .table(relation.getSourceTable().getPhysicalName())
                    .map(table -> table.foreignKeys().containsKey(constraintName))
                    .orElse(false);
            if (alreadyPresent) {
                continue;
            }
            // Deferred to its own phase so the tables and columns it mentions certainly exist,
            // whatever order they were created in within this same plan.
            statements.add(DdlStatement.safe(
                    DdlPhase.ADD_CONSTRAINT,
                    renderer.addForeignKey(relation, sourceField),
                    "Link %s.%s to %s on delete %s"
                            .formatted(
                                    relation.getSourceTable().getPhysicalName(),
                                    sourceField.getColumnName(),
                                    relation.getTargetTable().getPhysicalName(),
                                    relation.getOnDelete().toSql())));
        }

        planOrphanedForeignKeys(physical, declaredConstraints, statements);
    }

    private void planOrphanedForeignKeys(
            PhysicalSchema physical, Set<String> declaredConstraints, List<DdlStatement> statements) {

        for (PhysicalTable table : physical.tables().values()) {
            for (PhysicalForeignKey foreignKey : table.foreignKeys().values()) {
                if (declaredConstraints.contains(foreignKey.constraintName())
                        || foreignKey.constraintName().startsWith("fk_" + table.name() + "_source")
                        || foreignKey.constraintName().startsWith("fk_" + table.name() + "_target")) {
                    continue;
                }
                statements.add(DdlStatement.safe(
                        DdlPhase.DROP,
                        renderer.dropConstraint(table.name(), foreignKey.constraintName()),
                        "Remove link %s.%s to %s"
                                .formatted(table.name(), foreignKey.sourceColumn(), foreignKey.targetTable())));
            }
        }
    }

    // ------------------------------------------------------------ join tables

    private void planJoinTables(
            List<MetaRelation> relations, PhysicalSchema physical, List<DdlStatement> statements) {

        for (MetaRelation relation : relations) {
            if (relation.getKind() != RelationKind.MANY_TO_MANY
                    || physical.hasTable(relation.getJoinTableName())) {
                continue;
            }
            statements.add(DdlStatement.safe(
                    DdlPhase.JOIN_TABLE,
                    renderer.createJoinTable(relation),
                    "Create join table %s linking %s and %s"
                            .formatted(
                                    relation.getJoinTableName(),
                                    relation.getSourceTable().getPhysicalName(),
                                    relation.getTargetTable().getPhysicalName())));
        }
    }

    // --------------------------------------------------------- dropped tables

    private void planDroppedTables(
            List<MetaTable> declared,
            PhysicalSchema physical,
            Set<String> joinTableNames,
            PlanOptions options,
            List<DdlStatement> statements,
            Set<String> droppedEntities) {

        Set<String> declaredNames =
                declared.stream().map(MetaTable::getPhysicalName).collect(Collectors.toSet());

        for (String physicalName : physical.tableNames()) {
            if (declaredNames.contains(physicalName) || joinTableNames.contains(physicalName)) {
                continue;
            }

            List<String> dependents = dependentsOf(physicalName, physical, declaredNames);
            if (!dependents.isEmpty()) {
                statements.add(DdlStatement.blocked(
                        DdlPhase.DROP,
                        "Drop table %s".formatted(physicalName),
                        "Still referenced by %s. Remove those relations first."
                                .formatted(String.join(", ", dependents))));
                continue;
            }

            String key = inputKey(physicalName, null);
            if (options.isDropConfirmed(key)) {
                statements.add(DdlStatement.safe(
                        DdlPhase.DROP, renderer.dropTable(physicalName), "Drop table %s".formatted(physicalName)));
                droppedEntities.add(physicalName);
            } else {
                statements.add(DdlStatement.needsInput(
                        DdlPhase.DROP,
                        null,
                        "Drop table %s".formatted(physicalName),
                        "This destroys %d row(s) and every document attached to them. Confirm to proceed."
                                .formatted(schemaReader.countRows(physicalName)),
                        key));
            }
        }
    }

    /** Tables that still hold a foreign key pointing at the given table. */
    private List<String> dependentsOf(String physicalName, PhysicalSchema physical, Set<String> declaredNames) {
        return physical.tables().values().stream()
                .filter(table -> !table.name().equals(physicalName))
                .filter(table -> table.foreignKeys().values().stream()
                        .anyMatch(foreignKey -> foreignKey.targetTable().equals(physicalName)))
                .map(PhysicalTable::name)
                // A table that is itself about to disappear is not a reason to block.
                .filter(declaredNames::contains)
                .toList();
    }

    // ---------------------------------------------------------------- helpers

    static boolean isSystemColumn(String columnName) {
        return "id".equals(columnName) || "created_at".equals(columnName) || "updated_at".equals(columnName);
    }

    static String inputKey(String tableName, String columnName) {
        return columnName == null ? tableName : tableName + "." + columnName;
    }

    private static boolean hasDefault(MetaField field) {
        return field.getDefaultValue() != null && !field.getDefaultValue().isBlank();
    }
}
