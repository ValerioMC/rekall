package dev.rekall.mcp;

import tools.jackson.databind.JsonNode;
import tools.jackson.databind.node.NullNode;
import dev.rekall.mcp.protocol.JsonRpc;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.protocol.ProtocolVersion;
import dev.rekall.mcp.tool.ToolFailure;
import lombok.extern.slf4j.Slf4j;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;
import java.util.function.Supplier;
import java.util.stream.Collectors;

@RestController
@RequestMapping("/mcp")
@Slf4j
public class McpController {

    private static final String SERVER_NAME = "rekall";
    private static final String SERVER_VERSION = "0.1.0";

    private static final int CACHE_TTL_MS = 3_600_000;

    private static final String CACHE_SCOPE = "public";

    private static final String INSTRUCTIONS = """
            Rekall holds one user's companies, projects, tasks and markdown notes, and hands \
            back a whole working context in a single call. Anchor what you need as \
            `entity:value`, for example `project:vega task:report-builder`. Reading is \
            `rekall_context`. The only thing you may write is a task's wrapup, with \
            `rekall_wrapup`: what its implementation looks like now, replaced in place. \
            Nothing else here can be changed.""";

    private static final String BASE64_PREFIX = "=?base64?";

    private static final String BASE64_SUFFIX = "?=";

    private final Map<String, McpTool> tools;

    public McpController(List<McpTool> tools) {
        this.tools = tools.stream()
                .collect(Collectors.toMap(McpTool::name, Function.identity(), (a, b) -> a, LinkedHashMap::new));
        List<String> writing = tools.stream().filter(McpTool::writes).map(McpTool::name).toList();
        log.info("MCP server exposing {} tool(s): {}. Writes: {}",
                this.tools.size(), this.tools.keySet(), writing.isEmpty() ? "none" : writing);
    }

    @PostMapping(consumes = MediaType.APPLICATION_JSON_VALUE, produces = MediaType.APPLICATION_JSON_VALUE)
    public ResponseEntity<JsonRpc.Response> handle(
            @RequestHeader(value = "MCP-Protocol-Version", required = false) String versionHeader,
            @RequestHeader(value = "Mcp-Method", required = false) String methodHeader,
            @RequestHeader(value = "Mcp-Name", required = false) String nameHeader,
            @RequestBody JsonRpc.Request request) {

        if (!JsonRpc.VERSION.equals(request.jsonrpc())) {
            return ResponseEntity.ok(JsonRpc.Response.failure(
                    id(request), JsonRpc.INVALID_REQUEST, "Expected jsonrpc 2.0"));
        }
        if (request.method() == null || request.method().isBlank()) {
            return ResponseEntity.ok(JsonRpc.Response.failure(
                    id(request), JsonRpc.INVALID_REQUEST, "Request has no method"));
        }

        String declared = versionHeader != null ? versionHeader : metaProtocolVersion(request);
        ProtocolVersion version;
        if (declared == null) {
            version = ProtocolVersion.ASSUMED_WHEN_HEADER_ABSENT;
        } else {
            Optional<ProtocolVersion> known = ProtocolVersion.parse(declared);
            if (known.isEmpty()) {
                return unsupportedVersion(request, declared);
            }
            version = known.get();
        }

        return version.isModern()
                ? modern(request, versionHeader, methodHeader, nameHeader)
                : legacy(request);
    }

    private ResponseEntity<JsonRpc.Response> modern(
            JsonRpc.Request request, String versionHeader, String methodHeader, String nameHeader) {

        if (versionHeader == null) {
            return headerMismatch(request, "MCP-Protocol-Version header is required from 2026-07-28 on");
        }
        String metaVersion = metaProtocolVersion(request);
        if (metaVersion != null && !versionHeader.equals(metaVersion)) {
            return headerMismatch(request, "MCP-Protocol-Version '%s' does not match _meta '%s'"
                    .formatted(versionHeader, metaVersion));
        }
        if (request.isNotification()) {
            log.debug("MCP notification: {}", request.method());
            return ResponseEntity.accepted().build();
        }

        if (!request.method().equals(methodHeader)) {
            return headerMismatch(request, "Mcp-Method '%s' does not match body method '%s'"
                    .formatted(methodHeader, request.method()));
        }

        if ("tools/call".equals(request.method())) {
            String bodyName = textAt(request.params(), "name");
            String headerName;
            try {
                headerName = decodeHeaderValue(nameHeader);
            } catch (IllegalArgumentException e) {
                return headerMismatch(request, "Mcp-Name is not valid base64");
            }
            if (headerName == null || !headerName.equals(bodyName)) {
                return headerMismatch(request, "Mcp-Name '%s' does not match body name '%s'"
                        .formatted(nameHeader, bodyName));
            }
        }

        return guarded(request, () -> switch (request.method()) {
            case "server/discover" ->
                    ResponseEntity.ok(JsonRpc.Response.success(id(request), complete(cacheable(discover()))));
            case "tools/list" ->
                    ResponseEntity.ok(JsonRpc.Response.success(id(request), complete(cacheable(toolList()))));
            case "tools/call" ->
                    ResponseEntity.ok(JsonRpc.Response.success(id(request), complete(callTool(request.params()))));
            case "ping" -> ResponseEntity.ok(JsonRpc.Response.success(id(request), complete(Map.of())));
            default -> ResponseEntity.status(HttpStatus.NOT_FOUND).body(JsonRpc.Response.failure(
                    id(request), JsonRpc.METHOD_NOT_FOUND, "Unsupported method: " + request.method()));
        });
    }

    private ResponseEntity<JsonRpc.Response> legacy(JsonRpc.Request request) {
        if (request.isNotification()) {
            log.debug("MCP notification: {}", request.method());
            return ResponseEntity.accepted().build();
        }

        return guarded(request, () -> ResponseEntity.ok(switch (request.method()) {
            case "initialize" -> JsonRpc.Response.success(id(request), initialize(request));
            case "server/discover" -> JsonRpc.Response.success(id(request), complete(cacheable(discover())));
            case "tools/list" -> JsonRpc.Response.success(id(request), toolList());
            case "tools/call" -> JsonRpc.Response.success(id(request), callTool(request.params()));
            case "ping" -> JsonRpc.Response.success(id(request), Map.of());
            default -> JsonRpc.Response.failure(
                    id(request), JsonRpc.METHOD_NOT_FOUND, "Unsupported method: " + request.method());
        }));
    }

    private Map<String, Object> initialize(JsonRpc.Request request) {
        ProtocolVersion answer = ProtocolVersion.parse(textAt(request.params(), "protocolVersion"))
                .filter(version -> !version.isModern())
                .orElseGet(ProtocolVersion::latestLegacy);
        return Map.of(
                "protocolVersion", answer.wire(),
                "capabilities", Map.of("tools", Map.of()),
                "serverInfo", Map.of("name", SERVER_NAME, "version", SERVER_VERSION));
    }

    private Map<String, Object> discover() {
        return Map.of(
                "supportedVersions", ProtocolVersion.advertisedVersions(),
                "capabilities", Map.of("tools", Map.of()),
                "_meta", Map.of("io.modelcontextprotocol/serverInfo",
                        Map.of("name", SERVER_NAME, "version", SERVER_VERSION)),
                "instructions", INSTRUCTIONS);
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
        String name = params.get("name").asString();
        McpTool tool = tools.get(name);
        if (tool == null) {
            throw new IllegalArgumentException(
                    "Unknown tool '%s'. Available: %s".formatted(name, String.join(", ", tools.keySet())));
        }

        try {
            String text = tool.execute(params.get("arguments"));
            return content(text, false);
        } catch (ToolFailure | IllegalArgumentException e) {
            return content(e.getMessage(), true);
        }
    }

    private Map<String, Object> complete(Map<String, Object> result) {
        Map<String, Object> tagged = new LinkedHashMap<>(result);
        tagged.put("resultType", "complete");
        return tagged;
    }

    private Map<String, Object> cacheable(Map<String, Object> result) {
        Map<String, Object> annotated = new LinkedHashMap<>(result);
        annotated.put("ttlMs", CACHE_TTL_MS);
        annotated.put("cacheScope", CACHE_SCOPE);
        return annotated;
    }

    private Map<String, Object> content(String text, boolean isError) {
        return Map.of(
                "content", List.of(Map.of("type", "text", "text", text)),
                "isError", isError);
    }

    private ResponseEntity<JsonRpc.Response> guarded(
            JsonRpc.Request request, Supplier<ResponseEntity<JsonRpc.Response>> body) {
        try {
            return body.get();
        } catch (IllegalArgumentException e) {
            return ResponseEntity.ok(
                    JsonRpc.Response.failure(id(request), JsonRpc.INVALID_PARAMS, e.getMessage()));
        } catch (RuntimeException e) {
            log.error("MCP call {} failed", request.method(), e);
            return ResponseEntity.ok(JsonRpc.Response.failure(
                    id(request), JsonRpc.INTERNAL_ERROR, e.getClass().getSimpleName() + ": " + e.getMessage()));
        }
    }

    private ResponseEntity<JsonRpc.Response> unsupportedVersion(JsonRpc.Request request, String declared) {
        return ResponseEntity.badRequest().body(JsonRpc.Response.failure(
                id(request),
                JsonRpc.UNSUPPORTED_PROTOCOL_VERSION,
                "Unsupported protocol version",
                Map.of("supported", ProtocolVersion.advertisedVersions(), "requested", declared)));
    }

    private ResponseEntity<JsonRpc.Response> headerMismatch(JsonRpc.Request request, String detail) {
        return ResponseEntity.badRequest().body(JsonRpc.Response.failure(
                id(request), JsonRpc.HEADER_MISMATCH, "Header mismatch: " + detail));
    }

    private String decodeHeaderValue(String value) {
        if (value == null
                || value.length() < BASE64_PREFIX.length() + BASE64_SUFFIX.length()
                || !value.startsWith(BASE64_PREFIX)
                || !value.endsWith(BASE64_SUFFIX)) {
            return value;
        }
        String encoded = value.substring(BASE64_PREFIX.length(), value.length() - BASE64_SUFFIX.length());
        return new String(Base64.getDecoder().decode(encoded), StandardCharsets.UTF_8);
    }

    private String metaProtocolVersion(JsonRpc.Request request) {
        JsonNode params = request.params();
        if (params == null || !params.hasNonNull("_meta")) {
            return null;
        }
        return textAt(params.get("_meta"), "io.modelcontextprotocol/protocolVersion");
    }

    private String textAt(JsonNode node, String field) {
        return node != null && node.hasNonNull(field) ? node.get(field).asString() : null;
    }

    private JsonNode id(JsonRpc.Request request) {
        return request.id() == null ? NullNode.getInstance() : request.id();
    }
}
