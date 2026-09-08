package dev.rekall;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.test.web.server.LocalServerPort;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.springframework.web.client.RestClient;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.List;
import java.util.Map;
import java.util.function.BooleanSupplier;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * A hosted session end to end, over the same HTTP the console uses, against a stub that speaks a
 * slice of {@code claude --output-format stream-json}: start it on a task, watch a turn complete,
 * feed it a prompt, see the prompt and the reply land in the transcript, stop it.
 */
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class ClaudeSessionApiTest {

    private static final String STUB = """
            #!/bin/sh
            printf '%s\\n' '{"type":"system","subtype":"init","session_id":"stub-1"}'
            while IFS= read -r line; do
              printf '%s\\n' '{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"ack"}]}}'
              printf '%s\\n' '{"type":"result","subtype":"success","is_error":false,"result":"ack","duration_ms":1,"num_turns":1}'
            done
            """;

    private static final Path STUB_CLI = writeStub();

    @DynamicPropertySource
    static void claudeProperties(DynamicPropertyRegistry registry) {
        registry.add("rekall.claude.cli-path", STUB_CLI::toString);
        registry.add("rekall.claude.sweep-minutes", () -> "60");
    }

    @Autowired
    private JdbcTemplate jdbc;

    @LocalServerPort
    private int port;

    private RestClient rest;

    @BeforeEach
    void setUp() {
        rest = RestClient.builder()
                .baseUrl("http://localhost:" + port)
                .defaultStatusHandler(status -> true, (request, response) -> { })
                .build();
        jdbc.execute("DELETE FROM claude_message");
        jdbc.execute("DELETE FROM claude_session");
        jdbc.execute("DELETE FROM task_step");
        jdbc.execute("DELETE FROM task");
        jdbc.execute("DELETE FROM project");
        jdbc.execute("DELETE FROM company");
    }

    @Test
    @DisplayName("start, let a turn finish, prompt, read it back, stop")
    void fullRoundTrip() throws IOException {
        String taskId = aTaskWithFolder();

        Map<?, ?> started = post("/api/tasks/" + taskId + "/claude/sessions", Map.of("skipPermissions", true));
        String sessionId = String.valueOf(started.get("id"));
        assertThat(started.get("live")).isEqualTo(true);
        assertThat(String.valueOf(started.get("anchors"))).contains("task:");

        await(() -> "READY".equals(get("/api/claude/sessions/" + sessionId).get("status")));

        List<?> afterOpen = getList("/api/claude/sessions/" + sessionId + "/messages");
        assertThat(afterOpen).anySatisfy(row ->
                assertThat(((Map<?, ?>) row).get("role")).isEqualTo("ASSISTANT"));
        assertThat(get("/api/claude/sessions/" + sessionId).get("cliSessionId")).isEqualTo("stub-1");

        ResponseEntity<Map> prompt = rest.post().uri("/api/claude/sessions/" + sessionId + "/prompt")
                .body(Map.of("text", "what changed?"))
                .retrieve()
                .toEntity(Map.class);
        assertThat(prompt.getStatusCode()).isEqualTo(HttpStatus.ACCEPTED);

        await(() -> getList("/api/claude/sessions/" + sessionId + "/messages").stream()
                .anyMatch(row -> "USER".equals(((Map<?, ?>) row).get("role"))
                        && "what changed?".equals(((Map<?, ?>) row).get("content"))));
        await(() -> countRole(sessionId, "ASSISTANT") >= 2);

        Map<?, ?> stopped = post("/api/claude/sessions/" + sessionId + "/stop", Map.of());
        assertThat(stopped.get("status")).isEqualTo("EXITED");
        assertThat(stopped.get("live")).isEqualTo(false);

        ResponseEntity<Map> afterStop = rest.post().uri("/api/claude/sessions/" + sessionId + "/prompt")
                .body(Map.of("text", "still there?"))
                .retrieve()
                .toEntity(Map.class);
        assertThat(afterStop.getStatusCode()).isEqualTo(HttpStatus.CONFLICT);
    }

    @Test
    @DisplayName("a task whose project has no folder is refused, and no session row is left behind")
    void refusesWithoutAFolder() {
        String company = id(post("/api/companies", Map.of("name", "Acme")));
        String project = id(post("/api/projects", Map.of(
                "label", "vega", "title", "Vega", "status", "ACTIVE", "companyId", company)));
        String task = id(post("/api/tasks", Map.of(
                "label", "no-folder", "title", "No folder", "status", "TODO", "projectId", project)));

        ResponseEntity<Map> response = rest.post().uri("/api/tasks/" + task + "/claude/sessions")
                .body(Map.of("skipPermissions", true))
                .retrieve()
                .toEntity(Map.class);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
        assertThat(getList("/api/tasks/" + task + "/claude/sessions")).isEmpty();
    }

    // ---------------------------------------------------------------- helpers

    private String aTaskWithFolder() throws IOException {
        Path folder = Files.createTempDirectory("rekall-claude-test");
        String company = id(post("/api/companies", Map.of("name", "Acme")));
        String project = id(post("/api/projects", Map.of(
                "label", "vega", "title", "Vega", "status", "ACTIVE",
                "repoFolder", folder.toString(), "companyId", company)));
        return id(post("/api/tasks", Map.of(
                "label", "report-builder", "title", "Report builder", "status", "IN_PROGRESS",
                "projectId", project)));
    }

    private long countRole(String sessionId, String role) {
        return getList("/api/claude/sessions/" + sessionId + "/messages").stream()
                .filter(row -> role.equals(((Map<?, ?>) row).get("role")))
                .count();
    }

    private static void await(BooleanSupplier condition) {
        Instant deadline = Instant.now().plus(Duration.ofSeconds(10));
        while (Instant.now().isBefore(deadline)) {
            if (condition.getAsBoolean()) {
                return;
            }
            try {
                Thread.sleep(100);
            } catch (InterruptedException interrupted) {
                Thread.currentThread().interrupt();
                throw new IllegalStateException(interrupted);
            }
        }
        throw new AssertionError("condition not met within 10s");
    }

    private static Path writeStub() {
        try {
            Path stub = Files.createTempFile("fake-claude", ".sh");
            Files.writeString(stub, STUB);
            assertThat(stub.toFile().setExecutable(true)).isTrue();
            return stub;
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    private String id(Map<?, ?> body) {
        return String.valueOf(body.get("id"));
    }

    @SuppressWarnings("unchecked")
    private Map<?, ?> get(String path) {
        return rest.get().uri(path).retrieve().body(Map.class);
    }

    @SuppressWarnings("unchecked")
    private List<?> getList(String path) {
        return rest.get().uri(path).retrieve().body(List.class);
    }

    @SuppressWarnings("unchecked")
    private Map<?, ?> post(String path, Object body) {
        return rest.post().uri(path).body(body).retrieve().body(Map.class);
    }
}
