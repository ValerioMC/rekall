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
    }

    @JsonInclude(JsonInclude.Include.NON_NULL)
    public record Error(int code, String message, Map<String, Object> data) {
    }
}
