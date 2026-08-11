package dev.rekall.mcp.tool;

import tools.jackson.databind.JsonNode;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.protocol.ToolSchema;
import dev.rekall.mcp.render.SchemaRenderer;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;

import java.util.Map;

/**
 * Which anchors exist.
 *
 * <p>{@code rekall_context} takes an {@code entity:value} anchor and the entity names are
 * defined by the user, not by this code. Something has to be able to say what they currently
 * are, and this is it. It reads the meta-model and stops: it retrieves no records, which is why
 * it survived the reduction to a single retrieval tool.
 */
@Component
@RequiredArgsConstructor
public class SchemaTool implements McpTool {

    private final SchemaRenderer renderer;

    @Override
    public String name() {
        return "rekall_schema";
    }

    @Override
    public String description() {
        return """
               List every entity Rekall currently stores, with its description, fields and relations.
               The entity names are defined by the user and are not fixed, so this is how you find out
               which anchors `rekall_context` accepts. Call it only when an anchor was refused or when
               the user asks what exists: a well-formed anchor needs no lookup first.
               """;
    }

    @Override
    public Map<String, Object> inputSchema() {
        return ToolSchema.object().build();
    }

    @Override
    public String execute(JsonNode arguments) {
        return renderer.render();
    }
}
