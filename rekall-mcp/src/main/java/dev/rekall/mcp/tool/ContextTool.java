package dev.rekall.mcp.tool;

import tools.jackson.databind.JsonNode;
import dev.rekall.engine.data.QueryFilter;
import dev.rekall.engine.data.RecordView;
import dev.rekall.mcp.ReadOnlyDataAccess;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.protocol.ToolSchema;
import dev.rekall.mcp.render.RecordRenderer;
import dev.rekall.mcp.render.SchemaRenderer;
import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.RelationKind;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.Collectors;

/**
 * Loads the working context for a list of anchors, in a single call.
 *
 * <p>This is the only entry point into Rekall: a session starts with
 * {@code /rk project:stvv task:code-validator} and everything the answer needs arrives at once.
 *
 * <p>The tool is deliberately stupid. It resolves each anchor, loads the record with its
 * documents, follows the relations the meta-model declares, and stops. It does not interpret
 * natural language, does not infer which entity is a project and which is a task, and contains
 * no branch on any particular entity name. What to follow is data in the meta-model, not an
 * {@code if} in here, which is what lets tomorrow's anchors be {@code incident:ESA-4412}
 * without a code change.
 */
@Component
@RequiredArgsConstructor
public class ContextTool implements McpTool {

    /** Forward references arrive resolved. Depth stays at 1: a chain of two is already noise. */
    private static final int RESOLVE_DEPTH = 1;

    /** Ceiling on an inverse listing, so one heavily referenced record cannot flood the window. */
    private static final int MAX_REFERENCING = 50;

    private final ReadOnlyDataAccess data;
    private final RecordRenderer renderer;
    private final SchemaRenderer schema;

    @Override
    public String name() {
        return "rekall_context";
    }

    @Override
    public String description() {
        return """
               Load the full working context for one or more anchors in a single call.

               An anchor is `entity:value`, for example `project:stvv task:code-validator`. The
               entity part matches an entity's physical name, its label or any of its aliases,
               case-insensitively. The value matches that entity's display field. Quote a value
               that contains spaces: `environment:"kmaster14 / stvv-dev"`.

               A bare term with no `entity:` is looked up across every entity and is accepted only
               when exactly one record matches. On more than one match the candidates are returned
               with their entity and nothing is loaded.

               Each anchor brings back the record, the records it references resolved in full, the
               records that reference it as a list of anchors you can pass back, and every markdown
               document attached to it.
               """;
    }

    @Override
    public Map<String, Object> inputSchema() {
        return ToolSchema.object()
                .requiredString(
                        "anchors",
                        "Space-separated anchors, e.g. `project:stvv task:code-validator`. "
                                + "A single anchor is valid and loads that record alone.")
                .build();
    }

    @Override
    public String execute(JsonNode arguments) {
        List<Anchor> anchors = Anchor.parseAll(Arguments.of(arguments).requiredString("anchors"));
        StringBuilder out = new StringBuilder("# Context\n");
        for (Anchor anchor : anchors) {
            out.append('\n').append(render(resolve(anchor)));
        }
        return out.toString();
    }

    // ------------------------------------------------------------------ resolution

    private Resolved resolve(Anchor anchor) {
        if (anchor.isQualified()) {
            MetaTable entity = EntityResolution.require(data, schema, anchor.entityName());
            return new Resolved(entity, single(entity, anchor.value()));
        }
        return searchEveryEntity(anchor.value());
    }

    /** The qualified form: one entity, one value, no guessing left to do. */
    private RecordView single(MetaTable entity, String value) {
        List<RecordView> found = data.findByLabel(entity, value, RESOLVE_DEPTH);
        if (found.isEmpty()) {
            throw new ToolFailure("No %s matches '%s'".formatted(entity.getPhysicalName(), value));
        }
        if (found.size() > 1) {
            throw new ToolFailure("'%s' matches %d records of %s: %s. Pass a uuid to disambiguate."
                    .formatted(
                            value,
                            found.size(),
                            entity.getPhysicalName(),
                            found.stream().map(view -> view.id().toString()).collect(Collectors.joining(", "))));
        }
        return found.getFirst();
    }

    /**
     * The positional form. Every entity is tried and a single match is required: reporting the
     * candidates costs one more round trip, guessing between them costs a wrong context loaded
     * silently, which is the failure this application exists to avoid.
     */
    private Resolved searchEveryEntity(String value) {
        List<Resolved> matches = new ArrayList<>();
        for (MetaTable entity : data.schema().entities()) {
            for (RecordView view : data.findByLabel(entity, value, 0)) {
                matches.add(new Resolved(entity, view));
            }
        }
        if (matches.isEmpty()) {
            throw new ToolFailure("Nothing matches '%s'. Defined entities: %s."
                    .formatted(value, schema.availableEntityNames()));
        }
        if (matches.size() > 1) {
            throw new ToolFailure("'%s' matches %d records: %s. Qualify it as `entity:value`."
                    .formatted(value, matches.size(), matches.stream().map(Resolved::anchor).collect(Collectors.joining(", "))));
        }
        Resolved match = matches.getFirst();
        return new Resolved(
                match.entity(),
                data.findById(match.entity(), match.record().id(), RESOLVE_DEPTH).orElse(match.record()));
    }

    // ------------------------------------------------------------------ rendering

    private String render(Resolved resolved) {
        MetaTable entity = resolved.entity();
        RecordView record = resolved.record();
        return new StringBuilder("## ")
                .append(entity.getLabel())
                .append(": ")
                .append(record.label())
                .append("\n\n")
                .append(renderer.renderRecord(record, 0))
                .append(renderReferencing(entity, record))
                .append(renderer.renderDocuments(data.documentsFor(entity.getPhysicalName(), record.id())))
                .toString();
    }

    /**
     * Rule 3 of the requirements, applied generically: the inverse of a {@code MANY_TO_ONE} is
     * rendered as labels only. Loading those records in full is how a project drags every task it
     * ever had into the window, and the anchors printed alongside are enough to ask for one.
     */
    private String renderReferencing(MetaTable entity, RecordView record) {
        StringBuilder out = new StringBuilder();
        for (MetaRelation relation : data.schema().incomingRelations(entity.getPhysicalName())) {
            if (relation.getKind() != RelationKind.MANY_TO_ONE) {
                continue;
            }
            Optional<MetaTable> source = data.schema().entity(relation.getSourceTable().getPhysicalName());
            Optional<String> column = source.flatMap(table -> columnOf(table, relation.getSourceFieldId()));
            if (source.isEmpty() || column.isEmpty()) {
                continue;
            }
            out.append(renderReferencingRecords(source.get(), column.get(), record.id()));
        }
        return out.toString();
    }

    private String renderReferencingRecords(MetaTable source, String column, UUID targetId) {
        QueryFilter filter = QueryFilter.where(QueryFilter.Condition.eq(column, targetId));
        List<RecordView> referencing = data.query(source, filter, 0);
        if (referencing.isEmpty()) {
            return "";
        }
        StringBuilder out = new StringBuilder("\n#### ").append(source.getLabelPlural()).append('\n');
        referencing.stream()
                .limit(MAX_REFERENCING)
                .forEach(view -> out.append("- `")
                        .append(source.getPhysicalName())
                        .append(':')
                        .append(view.label())
                        .append("`\n"));

        long total = data.count(source, filter);
        if (total > MAX_REFERENCING) {
            out.append("[truncated: %d of %d shown]\n".formatted(MAX_REFERENCING, total));
        }
        return out.toString();
    }

    private Optional<String> columnOf(MetaTable entity, UUID fieldId) {
        if (fieldId == null) {
            return Optional.empty();
        }
        return entity.getFields().stream()
                .filter(field -> fieldId.equals(field.getId()))
                .map(MetaField::getColumnName)
                .findFirst();
    }

    /** A record together with the entity it was found in, which is what the anchor syntax names. */
    private record Resolved(MetaTable entity, RecordView record) {

        String anchor() {
            return entity.getPhysicalName() + ":" + record.label();
        }
    }
}
