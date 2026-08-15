package dev.rekall.mcp.protocol;

import com.fasterxml.jackson.annotation.JsonInclude;
import tools.jackson.databind.JsonNode;

import java.util.Map;

/**
 * The slice of JSON-RPC 2.0 that MCP over HTTP actually uses.
 *
 * <p>Hand-rolled rather than pulled from a library on purpose. The surface is three methods
 * wide, the transport is one POST endpoint, and owning it removes a dependency whose
 * compatibility with Spring Boot 4.1 would otherwise have to be tracked release by release.
 */
public final class JsonRpc {

    public static final String VERSION = "2.0";

    /** Error codes defined by JSON-RPC 2.0. */
    public static final int PARSE_ERROR = -32700;

    public static final int INVALID_REQUEST = -32600;
    public static final int METHOD_NOT_FOUND = -32601;
    public static final int INVALID_PARAMS = -32602;
    public static final int INTERNAL_ERROR = -32603;

    /*
     * Codes MCP defines on top of JSON-RPC, from the sub-range the specification reserves for
     * the protocol itself. Both are 2026-07-28 and carry an HTTP 400 alongside them.
     */

    /** A mirrored header disagrees with the body it was mirrored from, or is missing. */
    public static final int HEADER_MISMATCH = -32020;

    /** The revision the request declared is one this server does not speak. */
    public static final int UNSUPPORTED_PROTOCOL_VERSION = -32022;

    private JsonRpc() {
    }

    @JsonInclude(JsonInclude.Include.NON_NULL)
    public record Request(String jsonrpc, JsonNode id, String method, JsonNode params) {

        /**
         * A request without an id is a notification: the client is not waiting for an answer and
         * the server must not send one.
         */
        public boolean isNotification() {
            return id == null || id.isNull();
        }
    }

    @JsonInclude(JsonInclude.Include.NON_NULL)
    public record Response(String jsonrpc, JsonNode id, Object result, Error error) {

        public static Response success(JsonNode id, Object result) {
            return new Response(VERSION, id, result, null);
        }

        public static Response failure(JsonNode id, int code, String message) {
            return new Response(VERSION, id, null, new Error(code, message, null));
        }

        /**
         * A failure carrying structured data. The version errors are the reason this exists: a
         * client that asked for a revision this server does not speak needs the list of the ones
         * it does, in a field it can read, rather than a sentence it would have to parse.
         */
        public static Response failure(JsonNode id, int code, String message, Map<String, Object> data) {
            return new Response(VERSION, id, null, new Error(code, message, data));
        }
    }

    @JsonInclude(JsonInclude.Include.NON_NULL)
    public record Error(int code, String message, Map<String, Object> data) {
    }
}
