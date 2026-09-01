package dev.rekall.mcp;

import dev.rekall.mcp.protocol.JsonRpc;
import dev.rekall.mcp.protocol.McpTool;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;

import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.List;
import java.util.Map;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The protocol edge of the MCP server.
 *
 * <p>Two eras meet on one endpoint, and the failure they can produce is silent: a client that
 * gets served under the wrong era does not crash, it quietly sees no tools. So every case here
 * asserts the era a request was answered in, not only that an answer came back.
 */
class McpProtocolTest {

    private static final String MODERN = "2026-07-28";

    private static final JsonMapper JSON = JsonMapper.builder().build();

    private final McpController controller = new McpController(List.of(new StubTool()));

    // ------------------------------------------------------------------ legacy era

    @Nested
    @DisplayName("the handshake era")
    class Legacy {

        @Test
        @DisplayName("initialize is answered with the revision the client asked for")
        void echoesTheRequestedRevision() {
            var response = initialize("2025-06-18");

            assertThat(result(response)).containsEntry("protocolVersion", "2025-06-18");
        }

        @Test
        @DisplayName("a revision this server does not speak is answered with the newest handshake one")
        void fallsBackToTheNewestLegacyRevision() {
            // 2026-07-28 has no handshake, so a client that asks for it here cannot be served it.
            assertThat(result(initialize(MODERN))).containsEntry("protocolVersion", "2025-11-25");
            assertThat(result(initialize(null))).containsEntry("protocolVersion", "2025-11-25");
        }

        @Test
        @DisplayName("the revision this endpoint used to pin is still served")
        void stillServesTheOldPinnedRevision() {
            // The client registered before the dual-era split opens with exactly this.
            assertThat(result(initialize("2024-11-05"))).containsEntry("protocolVersion", "2024-11-05");
        }

        @Test
        @DisplayName("a request with no headers at all is read as a handshake-era request")
        void noHeadersMeansLegacy() {
            var response = call(null, null, null, request(1, "tools/list", null));

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
            assertThat(result(response)).containsKey("tools");
        }

        @Test
        @DisplayName("an unknown method is a JSON-RPC error on a 200, as this era expects")
        void unknownMethodStaysOn200() {
            var response = call(null, null, null, request(1, "resources/list", null));

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
            assertThat(error(response).code()).isEqualTo(JsonRpc.METHOD_NOT_FOUND);
        }

        private ResponseEntity<JsonRpc.Response> initialize(String requested) {
            JsonNode params = requested == null ? null : json("{\"protocolVersion\":\"%s\"}".formatted(requested));
            return call(null, null, null, request(1, "initialize", params));
        }
    }

    // ------------------------------------------------------------------ modern era

    @Nested
    @DisplayName("the stateless era")
    class Modern {

        @Test
        @DisplayName("tools/list is served when the headers agree with the body")
        void servesAWellFormedRequest() {
            var response = call(MODERN, "tools/list", null, request(1, "tools/list", null));

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
            assertThat(result(response)).containsKey("tools");
        }

        @Test
        @DisplayName("tools/call is served when Mcp-Name matches the tool in the body")
        void servesAToolCall() {
            var response = call(MODERN, "tools/call", "rekall_context", toolCall());

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
            assertThat(result(response)).containsEntry("isError", false);
        }

        @Test
        @DisplayName("a base64-wrapped Mcp-Name is decoded before it is compared")
        void decodesAWrappedName() {
            String wrapped = "=?base64?%s?=".formatted(
                    Base64.getEncoder().encodeToString("rekall_context".getBytes(StandardCharsets.UTF_8)));

            var response = call(MODERN, "tools/call", wrapped, toolCall());

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
            assertThat(result(response)).containsEntry("isError", false);
        }

        @Test
        @DisplayName("the era is refused when the version arrives only in the body")
        void refusesAMissingVersionHeader() {
            JsonNode params = json("{\"_meta\":{\"io.modelcontextprotocol/protocolVersion\":\"%s\"}}"
                    .formatted(MODERN));

            var response = call(null, "tools/list", null, request(1, "tools/list", params));

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
            assertThat(error(response).code()).isEqualTo(JsonRpc.HEADER_MISMATCH);
        }

        @Test
        @DisplayName("a header that disagrees with _meta is refused rather than picked between")
        void refusesAVersionDisagreement() {
            JsonNode params = json(
                    "{\"_meta\":{\"io.modelcontextprotocol/protocolVersion\":\"2025-06-18\"}}");

            var response = call(MODERN, "tools/list", null, request(1, "tools/list", params));

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
            assertThat(error(response).code()).isEqualTo(JsonRpc.HEADER_MISMATCH);
        }

        @Test
        @DisplayName("Mcp-Method has to match the method in the body")
        void refusesAMethodDisagreement() {
            var response = call(MODERN, "tools/list", null, request(1, "tools/call", null));

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
            assertThat(error(response).code()).isEqualTo(JsonRpc.HEADER_MISMATCH);
        }

        @Test
        @DisplayName("a missing Mcp-Method is refused, not assumed from the body")
        void refusesAMissingMethodHeader() {
            var response = call(MODERN, null, null, request(1, "tools/list", null));

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
            assertThat(error(response).code()).isEqualTo(JsonRpc.HEADER_MISMATCH);
        }

        @Test
        @DisplayName("tools/call without Mcp-Name is refused")
        void refusesAMissingNameHeader() {
            var response = call(MODERN, "tools/call", null, toolCall());

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
            assertThat(error(response).code()).isEqualTo(JsonRpc.HEADER_MISMATCH);
        }

        @Test
        @DisplayName("an Mcp-Name naming a different tool than the body is refused")
        void refusesANameDisagreement() {
            // The point of the check: a gateway routing on the header and this server executing
            // on the body would otherwise be acting on two different requests.
            var response = call(MODERN, "tools/call", "something_else", toolCall());

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
            assertThat(error(response).code()).isEqualTo(JsonRpc.HEADER_MISMATCH);
        }

        @Test
        @DisplayName("tools/list carries the cache annotations without which it is rejected whole")
        void toolListCarriesItsCacheAnnotations() {
            // Not defaulted client-side the way server/discover's are: a list missing either of
            // these is thrown away, and a session that cannot read the list registers no tools.
            Map<String, Object> result = result(call(MODERN, "tools/list", null, request(1, "tools/list", null)));

            assertThat(result).containsKey("tools");
            assertThat(result.get("ttlMs")).asInstanceOf(
                    org.assertj.core.api.InstanceOfAssertFactories.INTEGER).isNotNegative();
            assertThat(result).containsEntry("cacheScope", "public");
        }

        @Test
        @DisplayName("every result says which kind of result it is, or the client throws it away")
        void everyResultCarriesItsKind() {
            // A missing resultType is not read as complete in this era, it is rejected outright,
            // and a tools/list rejected is a session that sees no tools and says nothing.
            assertThat(result(call(MODERN, "tools/list", null, request(1, "tools/list", null))))
                    .containsEntry("resultType", "complete");
            assertThat(result(call(MODERN, "tools/call", "rekall_context", toolCall())))
                    .containsEntry("resultType", "complete");
            assertThat(result(call(MODERN, "ping", null, request(1, "ping", null))))
                    .containsEntry("resultType", "complete");
            assertThat(result(call(MODERN, "server/discover", null, request(1, "server/discover", null))))
                    .containsEntry("resultType", "complete");
        }

        @Test
        @DisplayName("initialize is an unknown method here, answered on a 404")
        void initializeIsGone() {
            var response = call(MODERN, "initialize", null, request(1, "initialize", null));

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.NOT_FOUND);
            assertThat(error(response).code()).isEqualTo(JsonRpc.METHOD_NOT_FOUND);
        }

        @Test
        @DisplayName("an unknown method is a 404, so a probe can tell this from a wrong address")
        void unknownMethodIs404() {
            var response = call(MODERN, "resources/list", null, request(1, "resources/list", null));

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.NOT_FOUND);
            assertThat(error(response).code()).isEqualTo(JsonRpc.METHOD_NOT_FOUND);
        }

        private JsonRpc.Request toolCall() {
            return request(1, "tools/call",
                    json("{\"name\":\"rekall_context\",\"arguments\":{\"anchors\":\"project:vega\"}}"));
        }
    }

    // ------------------------------------------------------------------ across both eras

    @Test
    @DisplayName("a revision this server does not speak comes back with the ones it does")
    void refusesAnUnknownRevision() {
        var response = call("1900-01-01", "tools/list", null, request(1, "tools/list", null));

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
        JsonRpc.Error error = error(response);
        assertThat(error.code()).isEqualTo(JsonRpc.UNSUPPORTED_PROTOCOL_VERSION);
        assertThat(error.data()).containsEntry("requested", "1900-01-01");
        assertThat(error.data().get("supported")).asInstanceOf(
                        org.assertj.core.api.InstanceOfAssertFactories.list(String.class))
                .contains(MODERN, "2025-11-25")
                // Never offered: this server has never spoken that revision's HTTP+SSE transport.
                .doesNotContain("2024-11-05");
    }

    @Test
    @DisplayName("server/discover answers in both eras, because it is what a probe sends")
    void discoverAnswersInBothEras() {
        for (var response : List.of(
                call(MODERN, "server/discover", null, request(1, "server/discover", null)),
                call(null, null, null, request(1, "server/discover", null)))) {

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
            Map<String, Object> result = result(response);
            assertThat(result).containsEntry("resultType", "complete").containsKey("ttlMs");
            assertThat(result.get("supportedVersions")).asInstanceOf(
                    org.assertj.core.api.InstanceOfAssertFactories.list(String.class)).contains(MODERN);
            assertThat(result.get("_meta")).asInstanceOf(
                            org.assertj.core.api.InstanceOfAssertFactories.map(String.class, Object.class))
                    .containsKey("io.modelcontextprotocol/serverInfo");
        }
    }

    @Test
    @DisplayName("a notification is accepted with no body, in either era")
    void notificationsGetNoBody() {
        for (var response : List.of(
                // No Mcp-Method on the first: the stateless revision requires that header on a
                // request and says nothing about a notification, so one must not be demanded.
                call(MODERN, null, null, request(null, "notifications/initialized", null)),
                call(null, null, null, request(null, "notifications/initialized", null)))) {

            assertThat(response.getStatusCode()).isEqualTo(HttpStatus.ACCEPTED);
            assertThat(response.getBody()).isNull();
        }
    }

    @Test
    @DisplayName("a body that is not JSON-RPC 2.0 is refused before any of this runs")
    void refusesForeignEnvelopes() {
        var response = controller.handle(null, null, null,
                new JsonRpc.Request("1.0", JSON.valueToTree(1), "tools/list", null));

        assertThat(error(response).code()).isEqualTo(JsonRpc.INVALID_REQUEST);
    }

    // ------------------------------------------------------------------ helpers

    private ResponseEntity<JsonRpc.Response> call(
            String versionHeader, String methodHeader, String nameHeader, JsonRpc.Request request) {
        return controller.handle(versionHeader, methodHeader, nameHeader, request);
    }

    private static JsonRpc.Request request(Integer id, String method, JsonNode params) {
        return new JsonRpc.Request(
                JsonRpc.VERSION, id == null ? null : JSON.valueToTree(id), method, params);
    }

    private static JsonNode json(String raw) {
        return JSON.readTree(raw);
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> result(ResponseEntity<JsonRpc.Response> response) {
        assertThat(response.getBody()).isNotNull();
        assertThat(response.getBody().error()).isNull();
        return (Map<String, Object>) response.getBody().result();
    }

    private static JsonRpc.Error error(ResponseEntity<JsonRpc.Response> response) {
        assertThat(response.getBody()).isNotNull();
        assertThat(response.getBody().error()).isNotNull();
        return response.getBody().error();
    }

    /** Stands in for {@code ContextTool}: this test is about the envelope, not what it carries. */
    private static final class StubTool implements McpTool {

        @Override
        public String name() {
            return "rekall_context";
        }

        @Override
        public String description() {
            return "Load a working context by anchor.";
        }

        @Override
        public Map<String, Object> inputSchema() {
            return Map.of("type", "object");
        }

        @Override
        public String execute(JsonNode arguments) {
            return "# Context";
        }
    }
}
