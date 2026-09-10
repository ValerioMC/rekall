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
import org.springframework.web.socket.BinaryMessage;
import org.springframework.web.socket.CloseStatus;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.client.standard.StandardWebSocketClient;
import org.springframework.web.socket.handler.AbstractWebSocketHandler;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.List;
import java.util.Map;
import java.util.function.BooleanSupplier;

import static org.assertj.core.api.Assertions.assertThat;

/** The in-app terminal end to end, against a stub that stands in for the interactive {@code claude} TUI. */
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class TerminalApiTest {

    private static final String BANNER = "rekall-terminal-stub-ready";

    private static final String STUB = """
            #!/bin/sh
            echo "%s"
            while IFS= read -r line; do
              echo "got:$line"
            done
            """.formatted(BANNER);

    private static final Path STUB_CLI = writeStub();

    @DynamicPropertySource
    static void terminalProperties(DynamicPropertyRegistry registry) {
        registry.add("rekall.claude.cli-path", STUB_CLI::toString);
        registry.add("rekall.terminal.sweep-minutes", () -> "60");
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
        jdbc.execute("DELETE FROM task_step");
        jdbc.execute("DELETE FROM task");
        jdbc.execute("DELETE FROM project");
        jdbc.execute("DELETE FROM company");
    }

    @AfterEach
    void closeLiveTerminals() {
        for (Object row : getList("/api/terminals")) {
            rest.delete().uri("/api/terminals/" + ((Map<?, ?>) row).get("id")).retrieve().toBodilessEntity();
        }
    }

    @Test
    @DisplayName("open, connect the pipe, see the banner and an echoed line, close")
    void fullRoundTrip() throws Exception {
        String taskId = aTaskWithFolder();

        Map<?, ?> opened = post("/api/tasks/" + taskId + "/terminals", Map.of("skipPermissions", true));
        String id = String.valueOf(opened.get("id"));
        assertThat(opened.get("live")).isEqualTo(true);
        assertThat(String.valueOf(opened.get("anchors"))).contains("task:");
        assertThat(getList("/api/terminals")).hasSize(1);
        assertThat(get("/api/terminals/" + id).get("id")).isEqualTo(id);

        Collector collector = new Collector();
        WebSocketSession session = new StandardWebSocketClient()
                .execute(collector, "ws://localhost:" + port + "/api/terminal/" + id + "/io")
                .get(Duration.ofSeconds(5).toMillis(), java.util.concurrent.TimeUnit.MILLISECONDS);

        try {
            await(() -> collector.text().contains(BANNER));
            session.sendMessage(new BinaryMessage(ByteBuffer.wrap("ping\n".getBytes(StandardCharsets.UTF_8))));
            await(() -> collector.text().contains("got:ping"));
        } finally {
            session.close(CloseStatus.NORMAL);
        }

        ResponseEntity<Void> deleted = rest.delete().uri("/api/terminals/" + id).retrieve().toBodilessEntity();
        assertThat(deleted.getStatusCode()).isEqualTo(HttpStatus.NO_CONTENT);
        assertThat(getList("/api/terminals")).isEmpty();
    }

    @Test
    @DisplayName("a task whose project has no folder is refused")
    void refusesWithoutAFolder() {
        String company = id(post("/api/companies", Map.of("name", "Acme")));
        String project = id(post("/api/projects", Map.of(
                "label", "vega", "title", "Vega", "status", "ACTIVE", "companyId", company)));
        String task = id(post("/api/tasks", Map.of(
                "label", "no-folder", "title", "No folder", "status", "TODO", "projectId", project)));

        ResponseEntity<Map> response = rest.post().uri("/api/tasks/" + task + "/terminals")
                .body(Map.of("skipPermissions", true))
                .retrieve()
                .toEntity(Map.class);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
        assertThat(getList("/api/terminals")).isEmpty();
    }

    @Test
    @DisplayName("connecting to an unknown terminal id closes the socket instead of hanging")
    void unknownTerminalIsRejected() throws Exception {
        Collector collector = new Collector();
        WebSocketSession session = new StandardWebSocketClient()
                .execute(collector, "ws://localhost:" + port + "/api/terminal/"
                        + java.util.UUID.randomUUID() + "/io")
                .get(5000, java.util.concurrent.TimeUnit.MILLISECONDS);

        await(() -> !session.isOpen());
    }

    // ---------------------------------------------------------------- helpers

    private static final class Collector extends AbstractWebSocketHandler {
        private final ByteArrayOutputStream bytes = new ByteArrayOutputStream();

        @Override
        protected synchronized void handleBinaryMessage(WebSocketSession session, BinaryMessage message) {
            ByteBuffer payload = message.getPayload();
            byte[] chunk = new byte[payload.remaining()];
            payload.get(chunk);
            bytes.writeBytes(chunk);
        }

        synchronized String text() {
            return bytes.toString(StandardCharsets.UTF_8);
        }
    }

    private String aTaskWithFolder() throws IOException {
        Path folder = Files.createTempDirectory("rekall-terminal-test");
        String company = id(post("/api/companies", Map.of("name", "Acme")));
        String project = id(post("/api/projects", Map.of(
                "label", "vega", "title", "Vega", "status", "ACTIVE",
                "repoFolder", folder.toString(), "companyId", company)));
        return id(post("/api/tasks", Map.of(
                "label", "report-builder", "title", "Report builder", "status", "IN_PROGRESS",
                "projectId", project)));
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
            Path stub = Files.createTempFile("fake-claude-tty", ".sh");
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
