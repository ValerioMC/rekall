package dev.rekall.mcp.tool;

import tools.jackson.databind.JsonNode;

/** Reads tool arguments out of a {@link JsonNode} with clear failures instead of nulls. */
public final class Arguments {

    private final JsonNode node;

    private Arguments(JsonNode node) {
        this.node = node;
    }

    public static Arguments of(JsonNode node) {
        return new Arguments(node);
    }

    public String requiredString(String name) {
        if (node == null || !node.hasNonNull(name) || node.get(name).asText().isBlank()) {
            throw new IllegalArgumentException("'%s' is required".formatted(name));
        }
        return node.get(name).asText();
    }
}
