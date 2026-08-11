package dev.rekall.api.service;

import dev.rekall.api.dto.MetaDtos;
import dev.rekall.engine.schema.SchemaRegistry;
import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.MetaTableStatus;
import dev.rekall.meta.domain.OnDeleteAction;
import dev.rekall.meta.domain.RelationKind;
import dev.rekall.meta.repository.MetaFieldRepository;
import dev.rekall.meta.repository.MetaRelationRepository;
import dev.rekall.meta.repository.MetaTableRepository;
import dev.rekall.meta.validation.IdentifierValidator;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.UUID;

/**
 * Edits the meta-model.
 *
 * <p>Nothing here touches the physical schema. Every change lands as a definition and moves the
 * entity to {@link MetaTableStatus#MODIFIED}; the database only changes when a plan is applied.
 * That separation is what makes the preview meaningful.
 */
@Service
@RequiredArgsConstructor
public class MetaModelService {

    private final MetaTableRepository tables;
    private final MetaFieldRepository fields;
    private final MetaRelationRepository relations;
    private final SchemaRegistry registry;

    @Transactional(readOnly = true)
    public List<MetaTable> listTables() {
        return tables.findAllWithFields();
    }

    @Transactional(readOnly = true)
    public MetaTable getTable(UUID id) {
        return tables.findWithFieldsById(id).orElseThrow(() -> new NotFoundException("entity", id));
    }

    @Transactional
    public MetaTable createTable(MetaDtos.CreateTableRequest request) {
        IdentifierValidator.validateTableName(request.physicalName());
        if (tables.existsByPhysicalName(request.physicalName())) {
            throw new ConflictException("An entity named '%s' already exists".formatted(request.physicalName()));
        }
        MetaTable table = new MetaTable(
                request.physicalName(), request.label(), request.labelPlural(), request.description());
        table.setAliases(toArray(request.aliases()));
        MetaTable saved = tables.saveAndFlush(table);
        registry.invalidate();
        return saved;
    }

    @Transactional
    public MetaTable updateTable(UUID id, MetaDtos.UpdateTableRequest request) {
        MetaTable table = getTable(id);
        table.setLabel(request.label());
        table.setLabelPlural(request.labelPlural());
        table.setDescription(request.description());
        table.setAliases(toArray(request.aliases()));

        if (request.displayFieldId() != null) {
            boolean belongsToTable = table.getFields().stream()
                    .anyMatch(field -> field.getId().equals(request.displayFieldId()));
            if (!belongsToTable) {
                throw new ConflictException("The display field must be one of the entity's own fields");
            }
        }
        table.setDisplayFieldId(request.displayFieldId());

        // Label and description are not physical, but the UI treats a single status per entity,
        // and marking it modified keeps "there are unapplied changes" honest.
        markModified(table);
        MetaTable saved = tables.saveAndFlush(table);
        registry.invalidate();
        return saved;
    }

    @Transactional
    public void deleteTable(UUID id) {
        MetaTable table = getTable(id);
        List<MetaRelation> touching = relations.findAllTouching(id);
        if (!touching.isEmpty()) {
            throw new ConflictException("Remove the %d relation(s) touching '%s' first"
                    .formatted(touching.size(), table.getPhysicalName()));
        }
        tables.delete(table);
        registry.invalidate();
    }

    @Transactional
    public MetaField addField(UUID tableId, MetaDtos.CreateFieldRequest request) {
        IdentifierValidator.validateColumnName(request.columnName());
        MetaTable table = getTable(tableId);
        if (table.findField(request.columnName()).isPresent()) {
            throw new ConflictException("'%s' already has a field named '%s'"
                    .formatted(table.getPhysicalName(), request.columnName()));
        }
        MetaField field =
                new MetaField(request.columnName(), request.label(), request.description(), request.type());
        applyFieldAttributes(
                field,
                request.nullable(),
                request.defaultValue(),
                request.length(),
                request.precision(),
                request.scale(),
                request.enumValues());

        table.addField(field);
        markModified(table);

        // Persisted directly rather than through the parent's cascade. save() on an entity that
        // already has an id goes through merge(), which persists a copy of any new child and
        // leaves the instance held here transient, so its generated id would never appear.
        MetaField saved = fields.saveAndFlush(field);
        registry.invalidate();
        return saved;
    }

    @Transactional
    public MetaField updateField(UUID fieldId, MetaDtos.UpdateFieldRequest request) {
        MetaField field = fields.findById(fieldId).orElseThrow(() -> new NotFoundException("field", fieldId));
        field.setLabel(request.label());
        field.setDescription(request.description());
        field.setType(request.type());
        applyFieldAttributes(
                field,
                request.nullable(),
                request.defaultValue(),
                request.length(),
                request.precision(),
                request.scale(),
                request.enumValues());

        markModified(field.getMetaTable());
        tables.saveAndFlush(field.getMetaTable());
        registry.invalidate();
        return field;
    }

    @Transactional
    public void deleteField(UUID fieldId) {
        MetaField field = fields.findById(fieldId).orElseThrow(() -> new NotFoundException("field", fieldId));
        relations.findBySourceFieldId(fieldId).ifPresent(relation -> {
            throw new ConflictException("This field backs a relation to '%s'. Delete the relation first."
                    .formatted(relation.getTargetTable().getPhysicalName()));
        });

        MetaTable table = field.getMetaTable();
        if (fieldId.equals(table.getDisplayFieldId())) {
            table.setDisplayFieldId(null);
        }
        table.removeField(field);
        markModified(table);
        tables.saveAndFlush(table);
        registry.invalidate();
    }

    @Transactional(readOnly = true)
    public List<MetaRelation> listRelations() {
        return relations.findAllWithTables();
    }

    @Transactional
    public MetaRelation createRelation(MetaDtos.CreateRelationRequest request) {
        MetaTable source = getTable(request.sourceTableId());
        MetaTable target = getTable(request.targetTableId());

        MetaRelation relation;
        if (request.kind() == RelationKind.MANY_TO_ONE) {
            if (request.sourceFieldId() == null) {
                throw new ConflictException("A many-to-one relation needs the reference field that carries it");
            }
            MetaField sourceField = source.getFields().stream()
                    .filter(field -> field.getId().equals(request.sourceFieldId()))
                    .findFirst()
                    .orElseThrow(() -> new ConflictException("The reference field must belong to the source entity"));
            if (!sourceField.getType().isReference()) {
                throw new ConflictException("'%s' is %s, and a relation needs a REFERENCE field"
                        .formatted(sourceField.getColumnName(), sourceField.getType()));
            }
            OnDeleteAction onDelete = request.onDelete() == null ? OnDeleteAction.RESTRICT : request.onDelete();
            if (onDelete == OnDeleteAction.SET_NULL && !sourceField.isNullable()) {
                throw new ConflictException("On delete set null needs '%s' to be optional"
                        .formatted(sourceField.getColumnName()));
            }
            relation = MetaRelation.manyToOne(
                    source, target, sourceField.getId(), onDelete, request.description());
        } else {
            String joinTable = request.joinTableName() == null || request.joinTableName().isBlank()
                    ? "rel_" + source.getPhysicalName() + "_" + target.getPhysicalName()
                    : request.joinTableName();
            IdentifierValidator.validateTableName(joinTable);
            relation = MetaRelation.manyToMany(source, target, joinTable, request.description());
        }

        MetaRelation saved = relations.saveAndFlush(relation);
        markModified(source);
        tables.saveAndFlush(source);
        registry.invalidate();
        return saved;
    }

    @Transactional
    public void deleteRelation(UUID id) {
        MetaRelation relation =
                relations.findById(id).orElseThrow(() -> new NotFoundException("relation", id));
        MetaTable source = relation.getSourceTable();
        relations.delete(relation);
        markModified(source);
        tables.saveAndFlush(source);
        registry.invalidate();
    }

    private void applyFieldAttributes(
            MetaField field,
            boolean nullable,
            String defaultValue,
            Integer length,
            Integer precision,
            Integer scale,
            List<String> enumValues) {

        field.setNullable(nullable);
        field.setDefaultValue(defaultValue);
        field.setLength(field.getType().usesLength() ? length : null);
        field.setPrecision(field.getType().usesPrecisionScale() ? precision : null);
        field.setScale(field.getType().usesPrecisionScale() ? scale : null);

        if (field.getType().usesEnumValues()) {
            if (enumValues == null || enumValues.isEmpty()) {
                throw new ConflictException("An enum field needs at least one allowed value");
            }
            field.setEnumValues(enumValues.toArray(String[]::new));
        } else {
            field.setEnumValues(null);
        }
    }

    /** A draft that was never applied stays a draft: only an applied entity becomes modified. */
    private void markModified(MetaTable table) {
        if (table.getStatus() == MetaTableStatus.APPLIED) {
            table.setStatus(MetaTableStatus.MODIFIED);
        }
    }

    private String[] toArray(List<String> values) {
        return values == null ? new String[0] : values.stream().filter(v -> !v.isBlank()).toArray(String[]::new);
    }
}
