package dev.rekall.mcp.render;

import dev.rekall.engine.schema.SchemaRegistry;
import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.RelationKind;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;

import java.util.List;
import java.util.stream.Collectors;

/**
 * Renders the meta-model as the map Claude navigates by.
 *
 * <p>This output is the entire semantic layer. There is no intent classifier and no embedding
 * index: a question reaches the right entity because the descriptions and aliases the user
 * wrote when defining it are in front of the model. Everything here is therefore optimised for
 * being read, not for being parsed.
 */
@Component
@RequiredArgsConstructor
public class SchemaRenderer {

    private final SchemaRegistry registry;

    public String render() {
        List<MetaTable> entities = registry.entities();
        if (entities.isEmpty()) {
            return """
                   No entities are defined yet.

                   Rekall stores whatever structure the user designs in its UI. Until they define
                   at least one entity there is nothing to query.
                   """;
        }

        StringBuilder out = new StringBuilder();
        out.append("# Rekall schema\n\n")
                .append("Entities currently defined. Each name below can be used as the entity part ")
                .append("of an anchor in `rekall_context`, for example `project:stvv`.\n");

        for (MetaTable entity : entities) {
            out.append('\n').append(renderEntity(entity));
        }
        return out.toString();
    }

    private String renderEntity(MetaTable entity) {
        StringBuilder out = new StringBuilder();
        out.append("## ").append(entity.getLabel()).append("  (`").append(entity.getPhysicalName()).append("`");

        String aliases = String.join(", ", entity.getAliases());
        if (!aliases.isBlank()) {
            out.append(", also known as: ").append(aliases);
        }
        out.append(")\n").append(entity.getDescription()).append('\n');

        entity.displayField()
                .ifPresent(field -> out.append("Records are identified by `")
                        .append(field.getColumnName())
                        .append("`.\n"));

        for (MetaField field : entity.getFields()) {
            out.append("- `").append(field.getColumnName()).append("` (").append(describeType(field));
            if (!field.isNullable()) {
                out.append(", required");
            }
            out.append(") ").append(field.getDescription()).append('\n');
        }

        for (MetaRelation relation : registry.outgoingRelations(entity.getPhysicalName())) {
            out.append("- ").append(describeRelation(relation, true)).append('\n');
        }
        for (MetaRelation relation : registry.incomingRelations(entity.getPhysicalName())) {
            if (relation.getSourceTable().getPhysicalName().equals(entity.getPhysicalName())) {
                continue;
            }
            out.append("- ").append(describeRelation(relation, false)).append('\n');
        }
        return out.toString();
    }

    private String describeType(MetaField field) {
        return switch (field.getType()) {
            case ENUM -> "one of: " + String.join(" | ", field.getEnumValues());
            case TAGS -> "list of tags";
            case REFERENCE -> "reference";
            case MARKDOWN -> "markdown text";
            case LONG_TEXT -> "long text";
            case TEXT -> "text";
            case INTEGER -> "number";
            case DECIMAL -> "decimal";
            case BOOLEAN -> "yes or no";
            case DATE -> "date";
            case TIMESTAMP -> "timestamp";
        };
    }

    /**
     * The inverse direction is described too, and marked as derived. A task knowing it belongs
     * to a project is only half the picture: without the other half, "which tasks are on STVV"
     * has no visible path from the project.
     */
    private String describeRelation(MetaRelation relation, boolean outgoing) {
        if (relation.getKind() == RelationKind.MANY_TO_MANY) {
            String other = outgoing
                    ? relation.getTargetTable().getPhysicalName()
                    : relation.getSourceTable().getPhysicalName();
            return "many `%s` (many-to-many). %s".formatted(other, relation.getDescription());
        }
        if (outgoing) {
            return "belongs to one `%s` (many-to-one). %s"
                    .formatted(relation.getTargetTable().getPhysicalName(), relation.getDescription());
        }
        return "has many `%s`, which reference it (derived). %s"
                .formatted(relation.getSourceTable().getPhysicalName(), relation.getDescription());
    }

    /** Comma-separated entity names, used in error messages when a name did not resolve. */
    public String availableEntityNames() {
        return registry.entities().stream()
                .map(MetaTable::getPhysicalName)
                .collect(Collectors.joining(", "));
    }
}
