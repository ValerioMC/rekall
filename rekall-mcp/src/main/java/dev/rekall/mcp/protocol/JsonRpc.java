package dev.rekall.mcp.protocol;

import com.fasterxml.jackson.annotation.JsonInclude;
import tools.jackson.databind.JsonNode;

import java.util.Map;

public final class JsonRpc {

    public static final String VERSION = "2.0";

    public static final int PARSE_ERROR = -32700;

    public static final int INVALID_REQUEST = -32600;
    public static final int METHOD_NOT_FOUND = -32601;
    public static final int INVALID_PARAMS = -32602;
    public static final int INTERNAL_ERROR = -32603;

    public static final int HEADER_MISMATCH = -32020;

    public static final int UNSUPPORTED_PROTOCOL_VERSION = -32022;

    private JsonRpc() {
    }

    @JsonInclude(JsonInclude.Include.NON_NULL)
    public record Request(String jsonrpc, JsonNode id, String method, JsonNode params) {

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

        public static Response failure(JsonNode id, int code, String message, Map<String, Object> data) {
            return new Response(VERSION, id, null, new Error(code, message, data));
        }
    }

    @JsonInclude(JsonInclude.Include.NON_NULL)
    public record Error(int code, String message, Map<String, Object> data) {
    }
}
