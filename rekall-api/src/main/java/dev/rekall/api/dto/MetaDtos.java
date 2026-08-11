package dev.rekall.api.dto;

import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaFieldType;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.MetaTableStatus;
import dev.rekall.meta.domain.OnDeleteAction;
import dev.rekall.meta.domain.RelationKind;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

import java.util.List;
import java.util.UUID;

/** Wire shapes for the schema designer. */
public final class MetaDtos {

    private MetaDtos() {
    }

    public record CreateTableRequest(
            @NotBlank String physicalName,
            @NotBlank String label,
            @NotBlank String labelPlural,
            @NotBlank String description,
            List<String> aliases) {
    }

    public record UpdateTableRequest(
            @NotBlank String label,
            @NotBlank String labelPlural,
            @NotBlank String description,
            List<String> aliases,
            UUID displayFieldId) {
    }

    public record CreateFieldRequest(
            @NotBlank String columnName,
            @NotBlank String label,
            @NotBlank String description,
            @NotNull MetaFieldType type,
            boolean nullable,
            String defaultValue,
            Integer length,
            Integer precision,
            Integer scale,
            List<String> enumValues) {
    }

    public record UpdateFieldRequest(
            @NotBlank String label,
            @NotBlank String description,
            @NotNull MetaFieldType type,
            boolean nullable,
            String defaultValue,
            Integer length,
            Integer precision,
            Integer scale,
            List<String> enumValues) {
    }

    public record CreateRelationRequest(
            @NotNull UUID sourceTableId,
            @NotNull UUID targetTableId,
            @NotNull RelationKind kind,
            UUID sourceFieldId,
            String joinTableName,
            OnDeleteAction onDelete,
            @NotBlank String description) {
    }

    public record TableResponse(
            UUID id,
            String physicalName,
            String label,
            String labelPlural,
            String description,
            List<String> aliases,
            UUID displayFieldId,
            MetaTableStatus status,
            List<FieldResponse> fields) {

        public static TableResponse from(MetaTable table) {
            return new TableResponse(
                    table.getId(),
                    table.getPhysicalName(),
                    table.getLabel(),
                    table.getLabelPlural(),
                    table.getDescription(),
                    List.of(table.getAliases()),
                    table.getDisplayFieldId(),
                    table.getStatus(),
                    table.getFields().stream().map(FieldResponse::from).toList());
        }
    }

    public record FieldResponse(
            UUID id,
            String columnName,
            String label,
            String description,
            MetaFieldType type,
            boolean nullable,
            String defaultValue,
            Integer length,
            Integer precision,
            Integer scale,
            List<String> enumValues,
            int position) {

        public static FieldResponse from(MetaField field) {
            return new FieldResponse(
                    field.getId(),
                    field.getColumnName(),
                    field.getLabel(),
                    field.getDescription(),
                    field.getType(),
                    field.isNullable(),
                    field.getDefaultValue(),
                    field.getLength(),
                    field.getPrecision(),
                    field.getScale(),
                    List.of(field.getEnumValues()),
                    field.getPosition());
        }
    }

    public record RelationResponse(
            UUID id,
            UUID sourceTableId,
            String sourceTableName,
            UUID targetTableId,
            String targetTableName,
            RelationKind kind,
            UUID sourceFieldId,
            String joinTableName,
            OnDeleteAction onDelete,
            String description) {

        public static RelationResponse from(MetaRelation relation) {
            return new RelationResponse(
                    relation.getId(),
                    relation.getSourceTable().getId(),
                    relation.getSourceTable().getPhysicalName(),
                    relation.getTargetTable().getId(),
                    relation.getTargetTable().getPhysicalName(),
                    relation.getKind(),
                    relation.getSourceFieldId(),
                    relation.getJoinTableName(),
                    relation.getOnDelete(),
                    relation.getDescription());
        }
    }
}
