package dev.rekall.mcp.tool;

import dev.rekall.mcp.ReadOnlyDataAccess;
import dev.rekall.mcp.render.SchemaRenderer;
import dev.rekall.meta.domain.MetaTable;

/** Turns the name a question used into an entity, or into a message that helps Claude retry. */
final class EntityResolution {

    private EntityResolution() {
    }

    /**
     * @throws ToolFailure listing what does exist. A bare "not found" would leave Claude
     *     guessing at a second name; naming the alternatives usually makes the retry correct.
     */
    static MetaTable require(ReadOnlyDataAccess data, SchemaRenderer schema, String name) {
        return data.resolveEntity(name)
                .orElseThrow(() -> new ToolFailure("No entity matches '%s'. Defined entities: %s. %s"
                        .formatted(
                                name,
                                schema.availableEntityNames(),
                                "Call rekall_schema for their descriptions and aliases.")));
    }
}
