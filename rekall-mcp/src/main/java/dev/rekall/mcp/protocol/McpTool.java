package dev.rekall.mcp.protocol;

import tools.jackson.databind.JsonNode;

import java.util.Map;

/**
 * One tool Claude can call.
 *
 * <p>Every implementation is read-only, held up by two things: this module's classpath carries
 * no controller and no write service, and every read runs under
 * {@code @Transactional(readOnly = true)}, so Hibernate will not flush. It used to be a
 * PostgreSQL role that held {@code SELECT} and nothing else, which the database enforced no
 * matter what the code did. On an embedded H2 file that would cost a second {@code DataSource}
 * and a duplicated set of repositories, so it was traded away deliberately;
 * {@code docs/DESIGN.md} §8 is the record of that.
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
