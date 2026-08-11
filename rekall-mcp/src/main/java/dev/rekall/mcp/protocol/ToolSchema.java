package dev.rekall.mcp.protocol;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/** Small builder for the JSON Schema fragments the tools declare. */
public final class ToolSchema {

    private final Map<String, Object> properties = new LinkedHashMap<>();
    private final List<String> required = new ArrayList<>();

    public static ToolSchema object() {
        return new ToolSchema();
    }

    public ToolSchema requiredString(String name, String description) {
        properties.put(name, Map.of("type", "string", "description", description));
        required.add(name);
        return this;
    }

    public Map<String, Object> build() {
        Map<String, Object> schema = new LinkedHashMap<>();
        schema.put("type", "object");
        schema.put("properties", properties);
        schema.put("required", List.copyOf(required));
        return schema;
    }
}
