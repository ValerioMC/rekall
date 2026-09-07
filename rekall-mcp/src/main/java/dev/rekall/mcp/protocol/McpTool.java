package dev.rekall.mcp.protocol;

import tools.jackson.databind.JsonNode;

import java.util.Map;

public interface McpTool {

    String name();

    String description();

    Map<String, Object> inputSchema();

    default boolean writes() {
        return false;
    }

    String execute(JsonNode arguments);
}
