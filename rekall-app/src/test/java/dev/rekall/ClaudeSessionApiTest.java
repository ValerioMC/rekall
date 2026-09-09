package dev.rekall;

import org.junit.jupiter.api.AfterEach;
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
            echo "$@" > "$0.args"
            printf '%s\\n' '{"type":"system","subtype":"init","session_id":"stub-1","model":"claude-sonnet-4-5-20250929"}'
            while IFS= read -r line; do
              printf '%s\\n' '{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"ack"}]}}'
              printf '%s\\n' '{"type":"result","subtype":"success","is_error":false,"result":"ack","duration_ms":1,"num_turns":1,"usage":{"input_tokens":10,"cache_read_input_tokens":1200,"output_tokens":40}}'
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

    @AfterEach
    void stopLiveProcesses() {
        for (Object row : getList("/api/claude/sessions")) {
            Map<?, ?> session = (Map<?, ?>) row;
            if (Boolean.TRUE.equals(session.get("live"))) {
                post("/api/claude/sessions/" + session.get("id") + "/stop", Map.of());
            }
        }
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
        assertThat(afterOpen).anySatisfy(row -> {
            assertThat(((Map<?, ?>) row).get("role")).isEqualTo("RESULT");
            assertThat(String.valueOf(((Map<?, ?>) row).get("meta"))).contains("\"totalTokens\":1250");
        });
        await(() -> "stub-1".equals(get("/api/claude/sessions/" + sessionId).get("cliSessionId")));
        await(() -> "claude-sonnet-4-5-20250929".equals(get("/api/claude/sessions/" + sessionId).get("model")));

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
    @DisplayName("the chosen model and effort are passed to claude and recorded on the session")
    void startsWithTheChosenModelAndEffort() throws IOException {
        String taskId = aTaskWithFolder();

        Map<?, ?> started = post("/api/tasks/" + taskId + "/claude/sessions",
                Map.of("skipPermissions", true, "model", "opus", "effort", "high"));
        String sessionId = String.valueOf(started.get("id"));

        await(() -> "READY".equals(get("/api/claude/sessions/" + sessionId).get("status")));

        assertThat(Files.readString(Path.of(STUB_CLI + ".args")))
                .contains("--model opus")
                .contains("--effort high");
        Map<?, ?> view = get("/api/claude/sessions/" + sessionId);
        assertThat(view.get("model")).isEqualTo("claude-sonnet-4-5-20250929");
        assertThat(view.get("effort")).isEqualTo("high");
    }

    @Test
    @DisplayName("with nothing chosen, neither --model nor --effort is passed and the account defaults stand")
    void startsWithoutAModelOrEffort() throws IOException {
        String taskId = aTaskWithFolder();

        Map<?, ?> started = post("/api/tasks/" + taskId + "/claude/sessions", Map.of("skipPermissions", true));
        String sessionId = String.valueOf(started.get("id"));

        await(() -> "READY".equals(get("/api/claude/sessions/" + sessionId).get("status")));

        assertThat(Files.readString(Path.of(STUB_CLI + ".args")))
                .doesNotContain("--model")
                .doesNotContain("--effort");
        assertThat(get("/api/claude/sessions/" + sessionId).get("effort")).isNull();
    }

    @Test
    @DisplayName("opening a session on a step marks it running, and stopping it releases the step")
    void aSessionDrivesTheStepItOpenedOn() throws IOException {
        String taskId = aTaskWithFolder();
        aStep(taskId, "Aggregate the rows");
        String secondStepId = aStep(taskId, "Write the tests");

        Map<?, ?> started = post("/api/tasks/" + taskId + "/claude/sessions",
                Map.of("skipPermissions", true, "stepId", secondStepId));
        String sessionId = String.valueOf(started.get("id"));
        assertThat(started.get("stepId")).isEqualTo(secondStepId);
        await(() -> "READY".equals(get("/api/claude/sessions/" + sessionId).get("status")));

        assertThat(stateOf(taskId, secondStepId)).isEqualTo("RUNNING");
        assertThat(stateOf(taskId, firstStepId(taskId))).isEqualTo("OPEN");

        post("/api/claude/sessions/" + sessionId + "/stop", Map.of());

        assertThat(stateOf(taskId, secondStepId)).isEqualTo("OPEN");
    }

    @Test
    @DisplayName("a second start on a live task reuses the process and retargets the running step")
    void secondStartReusesTheLiveProcess() throws IOException {
        String taskId = aTaskWithFolder();
        String stepA = aStep(taskId, "Aggregate the rows");
        String stepB = aStep(taskId, "Write the tests");

        Map<?, ?> first = post("/api/tasks/" + taskId + "/claude/sessions",
                Map.of("skipPermissions", true, "stepId", stepA));
        String sessionId = String.valueOf(first.get("id"));
        await(() -> "READY".equals(get("/api/claude/sessions/" + sessionId).get("status")));
        assertThat(stateOf(taskId, stepA)).isEqualTo("RUNNING");

        Map<?, ?> second = post("/api/tasks/" + taskId + "/claude/sessions",
                Map.of("skipPermissions", true, "stepId", stepB));

        assertThat(second.get("id")).isEqualTo(sessionId);
        assertThat(second.get("stepId")).isEqualTo(stepB);
        assertThat(getList("/api/tasks/" + taskId + "/claude/sessions")).hasSize(1);
        await(() -> "RUNNING".equals(stateOf(taskId, stepB)));
        assertThat(stateOf(taskId, stepA)).isEqualTo("OPEN");

        assertThat(getList("/api/claude/sessions/" + sessionId + "/messages")).anySatisfy(row -> {
            assertThat(((Map<?, ?>) row).get("role")).isEqualTo("SYSTEM");
            assertThat(String.valueOf(((Map<?, ?>) row).get("content"))).contains("Write the tests");
        });
    }

    @Test
    @DisplayName("clear reloads the context on the same session, no new process")
    void clearReloadsTheContext() throws IOException {
        String taskId = aTaskWithFolder();
        Map<?, ?> started = post("/api/tasks/" + taskId + "/claude/sessions", Map.of("skipPermissions", true));
        String sessionId = String.valueOf(started.get("id"));
        await(() -> "READY".equals(get("/api/claude/sessions/" + sessionId).get("status")));

        Map<?, ?> cleared = post("/api/claude/sessions/" + sessionId + "/clear", Map.of());
        assertThat(cleared.get("id")).isEqualTo(sessionId);
        assertThat(cleared.get("live")).isEqualTo(true);

        await(() -> getList("/api/claude/sessions/" + sessionId + "/messages").stream()
                .anyMatch(row -> "SYSTEM".equals(((Map<?, ?>) row).get("role"))
                        && String.valueOf(((Map<?, ?>) row).get("content")).contains("Context cleared")));
        assertThat(getList("/api/tasks/" + taskId + "/claude/sessions")).hasSize(1);
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

    private String aStep(String taskId, String title) {
        String stepId = id(post("/api/tasks/" + taskId + "/steps", Map.of("title", title)));
        rest.patch().uri("/api/steps/" + stepId).body(Map.of("draft", false))
                .retrieve().toEntity(Map.class);
        return stepId;
    }

    private String firstStepId(String taskId) {
        return String.valueOf(((Map<?, ?>) getList("/api/tasks/" + taskId + "/steps").getFirst()).get("id"));
    }

    private String stateOf(String taskId, String stepId) {
        return getList("/api/tasks/" + taskId + "/steps").stream()
                .map(row -> (Map<?, ?>) row)
                .filter(row -> stepId.equals(String.valueOf(row.get("id"))))
                .map(row -> String.valueOf(row.get("state")))
                .findFirst()
                .orElseThrow();
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
