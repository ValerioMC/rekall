package dev.rekall.mcp.protocol;

import tools.jackson.databind.JsonNode;

import java.util.Map;

/**
 * One tool Claude can call.
 *
 * <p>Reading is the default and writing is the exception, declared by {@link #writes()}. Every
 * read runs under {@code @Transactional(readOnly = true)}, so Hibernate will not flush, and the
 * one tool that does write reaches a service that can only replace the wrapup of a single task:
 * no entity is created, renamed or deleted from this module.
 *
 * <p>This used to be an absolute — a PostgreSQL role holding {@code SELECT} and nothing else,
 * enforced by the database no matter what the code did. It is now a shape rather than a
 * prohibition, because a wrapup Claude cannot put back is a wrapup nobody writes.
 * {@code docs/DESIGN.md} §8 is the record of both trades.
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

    /**
     * Whether calling this changes anything.
     *
     * <p>Declared rather than inferred, so the startup log names the write surface out loud
     * instead of describing every tool as read-only because that used to be true of all of them.
     */
    default boolean writes() {
        return false;
    }

    /** @return the text content returned to Claude */
    String execute(JsonNode arguments);
}
