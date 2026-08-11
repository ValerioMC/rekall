package dev.rekall.mcp.protocol;

import tools.jackson.databind.JsonNode;

import java.util.Map;

/**
 * One tool Claude can call.
 *
 * <p>Every implementation is read-only. That is not a convention to remember: the tools reach
 * the database through a datasource authenticated as {@code rekall_reader}, which holds
 * {@code SELECT} and nothing else, so a write would be refused by PostgreSQL rather than by a
 * check someone could forget to write.
 */
public interface McpTool {

    /** Name Claude calls, e.g. {@code rekall_context}. */
    String name();

    /**
     * What the tool does, in the terms a question would be asked in. This text is the only
     * thing standing between "Che progetti abbiamo attivi?" and the right call, so it describes
     * intent rather than mechanics.
     */
    String description();

    /** JSON Schema of the arguments object. */
    Map<String, Object> inputSchema();

    /** @return the text content returned to Claude */
    String execute(JsonNode arguments);
}
