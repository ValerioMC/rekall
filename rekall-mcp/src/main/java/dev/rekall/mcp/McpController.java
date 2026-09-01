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

/**
 * The MCP endpoint, spoken over one HTTP POST.
 *
 * <p>Registered with
 * {@code claude mcp add --transport http rekall http://localhost:47355/mcp}.
 *
 * <p>Two eras of the protocol arrive here and both are answered. A legacy client
 * ({@code 2025-11-25} and earlier) opens with {@code initialize} and is served the revision it
 * asked for. A modern client ({@code 2026-07-28}) sends no handshake at all: each request
 * carries its revision in {@code MCP-Protocol-Version}, its method in {@code Mcp-Method} and,
 * for {@code tools/call}, its tool name in {@code Mcp-Name}, and every one of those is checked
 * against the body before anything runs. See {@link ProtocolVersion} for why answering both
 * costs one branch.
 *
 * <p>Nothing here holds state between requests, in either era.
 *
 * <p>Reading is the whole of what this endpoint used to do. It now has one write as well:
 * {@code rekall_wrapup} replaces the wrapup of a single task, and nothing here can reach any
 * other row or column. Everything else runs through a service annotated
 * {@code @Transactional(readOnly = true)}, on a classpath that still carries no controller.
 * That is weaker again than the database role it once was; {@code docs/DESIGN.md} §8 records
 * both trades.
 */
@RestController
@RequestMapping("/mcp")
@Slf4j
public class McpController {

    private static final String SERVER_NAME = "rekall";
    private static final String SERVER_VERSION = "0.1.0";

    /**
     * How long a client may cache the answers that carry cache annotations, {@code server/discover}
     * and {@code tools/list}. An hour: both are constants in any given build, so the only thing a
     * stale copy can cost is one restart.
     */
    private static final int CACHE_TTL_MS = 3_600_000;

    /** Who a cached copy may be shared with. Nothing in either answer belongs to anyone. */
    private static final String CACHE_SCOPE = "public";

    /** Guidance carried by {@code server/discover}, for a client to display or pass on. */
    private static final String INSTRUCTIONS = """
            Rekall holds one user's companies, projects, tasks and markdown notes, and hands \
            back a whole working context in a single call. Anchor what you need as \
            `entity:value`, for example `project:vega task:report-builder`. Reading is \
            `rekall_context`. The only thing you may write is a task's wrapup, with \
            `rekall_wrapup`: what its implementation looks like now, replaced in place. \
            Nothing else here can be changed.""";

    /** The wrapper the specification puts around a header value that is not plain ASCII. */
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

        // The header is the modern era's answer; _meta is read too so that a request that
        // declares a revision only in its body is routed by what it says rather than silently
        // falling back, and can then be told which of the two the server actually needs.
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

    /**
     * The {@code 2026-07-28} era. No handshake and no session, so the request has to carry
     * everything; and the headers an intermediary may have routed on have to agree with the body
     * it forwarded, or that intermediary and this server would be acting on two different
     * requests.
     */
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
        // Ahead of the header checks below, because the revision that requires those headers
        // explicitly leaves what a notification must carry undefined. Refusing one for a header
        // the specification never asked it for would be this server's own rule.
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
            // 404 rather than 200, so a client probing an address can tell an MCP server that
            // does not know this method from something that is not an MCP endpoint at all. This
            // is also where `initialize` lands in this era, which is what tells a client that
            // opened with a handshake that it is talking to the wrong side of the split.
            default -> ResponseEntity.status(HttpStatus.NOT_FOUND).body(JsonRpc.Response.failure(
                    id(request), JsonRpc.METHOD_NOT_FOUND, "Unsupported method: " + request.method()));
        });
    }

    /**
     * The handshake era, {@code 2025-11-25} and earlier. Kept because the client registered
     * against this endpoint opens this way, and because a legacy client has no way to fall
     * forward: drop this branch and it has nothing to fall back to either.
     */
    private ResponseEntity<JsonRpc.Response> legacy(JsonRpc.Request request) {
        if (request.isNotification()) {
            log.debug("MCP notification: {}", request.method());
            return ResponseEntity.accepted().build();
        }

        return guarded(request, () -> ResponseEntity.ok(switch (request.method()) {
            case "initialize" -> JsonRpc.Response.success(id(request), initialize(request));
            // The one legacy-era result that still carries those, because they are part of the
            // shape `server/discover` is defined with rather than of the envelope around it.
            case "server/discover" -> JsonRpc.Response.success(id(request), complete(cacheable(discover())));
            case "tools/list" -> JsonRpc.Response.success(id(request), toolList());
            case "tools/call" -> JsonRpc.Response.success(id(request), callTool(request.params()));
            case "ping" -> JsonRpc.Response.success(id(request), Map.of());
            default -> JsonRpc.Response.failure(
                    id(request), JsonRpc.METHOD_NOT_FOUND, "Unsupported method: " + request.method());
        }));
    }

    /**
     * The legacy handshake. The revision the client asked for is echoed back when this server
     * speaks it, which is the whole of the negotiation the specification defines. Anything else
     * is answered with the newest revision that still has a handshake, and the client decides
     * whether it can live with that.
     */
    private Map<String, Object> initialize(JsonRpc.Request request) {
        ProtocolVersion answer = ProtocolVersion.parse(textAt(request.params(), "protocolVersion"))
                .filter(version -> !version.isModern())
                .orElseGet(ProtocolVersion::latestLegacy);
        return Map.of(
                "protocolVersion", answer.wire(),
                "capabilities", Map.of("tools", Map.of()),
                "serverInfo", Map.of("name", SERVER_NAME, "version", SERVER_VERSION));
    }

    /**
     * Mandatory from {@code 2026-07-28}: with the handshake gone, this is how a client learns
     * what the server speaks. Answered in both eras, because it is also the probe a dual-era
     * client sends to find out which era it has reached.
     */
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
            // Returned as tool content with isError rather than as a protocol error: this is a
            // result Claude can read and retry from, not a broken request.
            return content(e.getMessage(), true);
        }
    }

    /**
     * Tags a result with the kind of result it is, which every result carries from
     * {@code 2026-07-28} on. Always {@code complete} here: nothing this server does outlives the
     * response it is answering or asks the client for anything, so neither {@code task} nor
     * {@code input_required} can arise. The field is not optional in that era and its absence is
     * not read as {@code complete}: a result without it is rejected, and the tools go with it.
     */
    private Map<String, Object> complete(Map<String, Object> result) {
        Map<String, Object> tagged = new LinkedHashMap<>(result);
        tagged.put("resultType", "complete");
        return tagged;
    }

    /**
     * How long this answer may be held and who it may be shared with. Required on a list result
     * from {@code 2026-07-28} on, and required outright: unlike {@code server/discover}, whose
     * client-side schema defaults both, a {@code tools/list} missing either is rejected whole and
     * takes every tool in it down with it.
     */
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

    // ------------------------------------------------------------------ protocol plumbing

    /** One place where a tool blowing up becomes a JSON-RPC error rather than a 500. */
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

    /** Undoes the base64 wrapper the specification uses for a value that is not plain ASCII. */
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
