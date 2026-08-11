package dev.rekall.mcp;

import tools.jackson.databind.JsonNode;
import tools.jackson.databind.node.NullNode;
import dev.rekall.mcp.protocol.JsonRpc;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.tool.ToolFailure;
import lombok.extern.slf4j.Slf4j;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Function;
import java.util.stream.Collectors;

/**
 * The MCP endpoint, spoken over one HTTP POST.
 *
 * <p>Registered with {@code claude mcp add --transpot http rekall http://localhost:8080/mcp}.r
 * Every tool reachable from here reads through {@link ReadOnlyDataAccess} and therefore through
 * a database role that cannot write.
 */
@RestController
@RequestMapping("/mcp")
@Slf4j
public class McpController {

    /** MCP revision this server implements. */
    private static final String PROTOCOL_VERSION = "2024-11-05";

    private final Map<String, McpTool> tools;

    public McpController(List<McpTool> tools) {
        this.tools = tools.stream()
                .collect(Collectors.toMap(McpTool::name, Function.identity(), (a, b) -> a, LinkedHashMap::new));
        log.info("MCP server exposing {} read-only tool(s): {}", this.tools.size(), this.tools.keySet());
    }

    @PostMapping(consumes = MediaType.APPLICATION_JSON_VALUE, produces = MediaType.APPLICATION_JSON_VALUE)
    public ResponseEntity<JsonRpc.Response> handle(@RequestBody JsonRpc.Request request) {
        if (!JsonRpc.VERSION.equals(request.jsonrpc())) {
            return ResponseEntity.ok(JsonRpc.Response.failure(
                    id(request), JsonRpc.INVALID_REQUEST, "Expected jsonrpc 2.0"));
        }

        // A notification has no id and must get no response body, only an accepted status.
        if (request.isNotification()) {
            log.debug("MCP notification: {}", request.method());
            return ResponseEntity.accepted().build();
        }

        try {
            return ResponseEntity.ok(switch (request.method()) {
                case "initialize" -> JsonRpc.Response.success(id(request), initialize());
                case "tools/list" -> JsonRpc.Response.success(id(request), toolList());
                case "tools/call" -> JsonRpc.Response.success(id(request), callTool(request.params()));
                case "ping" -> JsonRpc.Response.success(id(request), Map.of());
                default -> JsonRpc.Response.failure(
                        id(request), JsonRpc.METHOD_NOT_FOUND, "Unsupported method: " + request.method());
            });
        } catch (IllegalArgumentException e) {
            return ResponseEntity.ok(
                    JsonRpc.Response.failure(id(request), JsonRpc.INVALID_PARAMS, e.getMessage()));
        } catch (RuntimeException e) {
            log.error("MCP call {} failed", request.method(), e);
            return ResponseEntity.ok(JsonRpc.Response.failure(
                    id(request), JsonRpc.INTERNAL_ERROR, e.getClass().getSimpleName() + ": " + e.getMessage()));
        }
    }

    private Map<String, Object> initialize() {
        return Map.of(
                "protocolVersion", PROTOCOL_VERSION,
                "capabilities", Map.of("tools", Map.of()),
                "serverInfo", Map.of("name", "rekall", "version", "0.1.0"));
    }

    private Map<String, Object> toolList() {
        List<Map<String, Object>> descriptors = tools.values().stream()
                .map(tool -> Map.of(
                        "name", (Object) tool.name(),
                        "description", tool.description(),
                        "inputSchema", tool.inputSchema()))
                .toList();
        return Map.of("tools", descriptors);
    }

    private Map<String, Object> callTool(JsonNode params) {
        if (params == null || !params.hasNonNull("name")) {
            throw new IllegalArgumentException("tools/call needs a 'name'");
        }
        String name = params.get("name").asText();
        McpTool tool = tools.get(name);
        if (tool == null) {
            throw new IllegalArgumentException(
                    "Unknown tool '%s'. Available: %s".formatted(name, String.join(", ", tools.keySet())));
        }

        try {
            String text = tool.execute(params.get("arguments"));
            return content(text, false);
        } catch (ToolFailure | IllegalArgumentException e) {
            // Returned as tool content with isError rather than as a protocol error: this is a
            // result Claude can read and retry from, not a broken request.
            return content(e.getMessage(), true);
        }
    }

    private Map<String, Object> content(String text, boolean isError) {
        return Map.of(
                "content", List.of(Map.of("type", "text", "text", text)),
                "isError", isError);
    }

    private JsonNode id(JsonRpc.Request request) {
        return request.id() == null ? NullNode.getInstance() : request.id();
    }
}
