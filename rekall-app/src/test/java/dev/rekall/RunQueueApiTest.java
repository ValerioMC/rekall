package dev.rekall;

import dev.rekall.claude.ClaudeUsageService;
import dev.rekall.claude.ClaudeUsageView;
import dev.rekall.claude.ClaudeUsageView.Limit;
import dev.rekall.claude.ClaudeUsageView.Severity;
import dev.rekall.domain.TaskStepState;
import dev.rekall.domain.review.TaskReviewService;
import dev.rekall.domain.step.TaskStepService;
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
import org.springframework.test.context.bean.override.mockito.MockitoBean;
import org.springframework.web.client.RestClient;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.function.BooleanSupplier;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.when;

/**
 * The run queue end to end: a stub stands in for the {@code claude} TUI, steps are claimed through
 * the same service the MCP tool uses, and the usage reading is whatever the test says it is.
 */
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class RunQueueApiTest {

    private static final String STUB = """
            #!/bin/sh
            echo "rekall-queue-stub-ready"
            while IFS= read -r line; do
              echo "got:$line"
            done
            """;

    private static final Path STUB_CLI = writeStub();
    private static final Instant SESSION_RESET = Instant.now().plus(Duration.ofHours(2));

    @DynamicPropertySource
    static void properties(DynamicPropertyRegistry registry) {
        registry.add("rekall.claude.cli-path", STUB_CLI::toString);
        registry.add("rekall.terminal.sweep-minutes", () -> "60");
        registry.add("rekall.run-queue.tick-seconds", () -> "1");
        registry.add("rekall.run-queue.settle-grace-seconds", () -> "0");
    }

    @MockitoBean
    private ClaudeUsageService usage;

    @Autowired
    private TaskStepService steps;

    @Autowired
    private TaskReviewService review;

    @Autowired
    private JdbcTemplate jdbc;

    @LocalServerPort
    private int port;

    private RestClient rest;
    private Path folder;
    private String projectId;

    @BeforeEach
    void setUp() throws IOException {
        rest = RestClient.builder()
                .baseUrl("http://localhost:" + port)
                .defaultStatusHandler(status -> true, (request, response) -> { })
                .build();
        rest.post().uri("/api/run-queue/stop").retrieve().toBodilessEntity();
        closeLiveTerminals();
        jdbc.execute("DELETE FROM run_queue_item");
        jdbc.execute("DELETE FROM run_queue");
        jdbc.execute("DELETE FROM task_step");
        jdbc.execute("DELETE FROM task");
        jdbc.execute("DELETE FROM project");
        jdbc.execute("DELETE FROM company");
        usageAt(10);

        folder = Files.createTempDirectory("rekall-queue-test");
        String company = id(post("/api/companies", Map.of("name", "Acme")));
        projectId = id(post("/api/projects", Map.of(
                "label", "vega", "title", "Vega", "status", "ACTIVE",
                "repoFolder", folder.toString(), "companyId", company)));
    }

    @AfterEach
    void tearDown() {
        rest.post().uri("/api/run-queue/stop").retrieve().toBodilessEntity();
        closeLiveTerminals();
    }

    @Test
    @DisplayName("runs each task in its own terminal, one after the other, and goes idle at the end")
    void runsInOrder() {
        UUID checklist = task("report-builder", "Report builder");
        openStep(checklist, "Build the export");
        openStep(checklist, "Expose the download");
        UUID stepless = task("fix-readme", "Fix readme");
        enqueue(checklist);
        enqueue(stepless);

        Map<?, ?> started = post("/api/run-queue/start", Map.of());
        assertThat(started.get("state")).isEqualTo("RUNNING");

        await(() -> itemState(0).equals("RUNNING") && terminalOn(checklist) != null);
        steps.transition("vega", "report-builder", "1", TaskStepState.CLAIMED);
        await(() -> itemState(0).equals("RUNNING"));
        assertThat(terminalOn(checklist)).as("one claim of two leaves the session working").isNotNull();

        steps.transition("vega", "report-builder", "2", TaskStepState.CLAIMED);
        await(() -> itemState(0).equals("FINISHED") && itemState(1).equals("RUNNING"));
        assertThat(terminalOn(checklist)).isNull();
        assertThat(terminalOn(stepless)).isNotNull();

        review.claimedByWrapup(stepless);
        await(() -> itemState(1).equals("FINISHED") && queue().get("state").equals("IDLE"));
        assertThat(terminalOn(stepless)).isNull();
    }

    @Test
    @DisplayName("a task with nothing open is skipped, and a session that ends early fails its item")
    void skipsAndFails() {
        UUID claimed = task("done-already", "Done already");
        openStep(claimed, "Only step");
        steps.transition("vega", "done-already", "1", TaskStepState.CLAIMED);
        UUID quitter = task("quitter", "Quitter");
        openStep(quitter, "Never claimed");
        enqueue(claimed);
        enqueue(quitter);

        post("/api/run-queue/start", Map.of());

        await(() -> itemState(0).equals("SKIPPED") && itemState(1).equals("RUNNING"));
        String terminal = terminalOn(quitter);
        assertThat(terminal).isNotNull();
        rest.delete().uri("/api/terminals/" + terminal).retrieve().toBodilessEntity();

        await(() -> itemState(1).equals("FAILED") && queue().get("state").equals("IDLE"));
        assertThat(item(1).get("detail").toString()).contains("ended before");
    }

    @Test
    @DisplayName("over the ceiling at the start, the queue holds until the session window resets")
    void holdsAtStart() {
        UUID task = task("report-builder", "Report builder");
        enqueue(task);
        put("/api/run-queue/settings", Map.of("ceilingPercent", 80, "skipPermissions", true));
        usageAt(85);

        post("/api/run-queue/start", Map.of());

        await(() -> queue().get("state").equals("HOLDING"));
        assertThat(terminalOn(task)).isNull();
        assertThat(itemState(0)).isEqualTo("QUEUED");
        Instant holdUntil = Instant.parse(queue().get("holdUntil").toString());
        assertThat(holdUntil).isAfter(SESSION_RESET);
        assertThat(queue().get("holdReason").toString()).contains("85%");
    }

    @Test
    @DisplayName("crossing the ceiling at a claim closes the session and puts the task back at the head")
    void pausesAtStepBoundary() {
        UUID task = task("report-builder", "Report builder");
        openStep(task, "Build the export");
        openStep(task, "Expose the download");
        enqueue(task);
        put("/api/run-queue/settings", Map.of("ceilingPercent", 80));

        post("/api/run-queue/start", Map.of());
        await(() -> terminalOn(task) != null);

        usageAt(82);
        steps.transition("vega", "report-builder", "2", TaskStepState.RUNNING);
        steps.transition("vega", "report-builder", "1", TaskStepState.CLAIMED);

        await(() -> queue().get("state").equals("HOLDING") && terminalOn(task) == null);
        assertThat(itemState(0)).isEqualTo("QUEUED");
        assertThat(item(0).get("detail").toString()).contains("ceiling");
        assertThat(steps.findByTask(task).get(1).state())
                .as("a step the session had started is left open for the next one")
                .isEqualTo(TaskStepState.OPEN);
    }

    @Test
    @DisplayName("a start time in the future schedules the queue; stop disarms it")
    void schedulesAndStops() {
        enqueue(task("report-builder", "Report builder"));

        Instant at = Instant.now().plus(Duration.ofHours(3));
        Map<?, ?> scheduled = post("/api/run-queue/start", Map.of("startAt", at.toString()));

        assertThat(scheduled.get("state")).isEqualTo("SCHEDULED");
        assertThat(Instant.parse(scheduled.get("startAt").toString())).isEqualTo(at);

        Map<?, ?> stopped = post("/api/run-queue/stop", Map.of());
        assertThat(stopped.get("state")).isEqualTo("IDLE");
        assertThat(stopped.get("startAt")).isNull();
    }

    @Test
    @DisplayName("the queue refuses a duplicate task, an empty start, a past time and a silly ceiling")
    void refusals() {
        ResponseEntity<Map> emptyStart = rest.post().uri("/api/run-queue/start").body(Map.of()).retrieve().toEntity(Map.class);
        assertThat(emptyStart.getStatusCode()).isEqualTo(HttpStatus.CONFLICT);

        UUID task = task("report-builder", "Report builder");
        enqueue(task);
        ResponseEntity<Map> twice = rest.post().uri("/api/run-queue/items")
                .body(Map.of("taskId", task.toString())).retrieve().toEntity(Map.class);
        assertThat(twice.getStatusCode()).isEqualTo(HttpStatus.CONFLICT);

        ResponseEntity<Map> past = rest.post().uri("/api/run-queue/start")
                .body(Map.of("startAt", Instant.now().minus(Duration.ofHours(1)).toString()))
                .retrieve().toEntity(Map.class);
        assertThat(past.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);

        ResponseEntity<Map> ceiling = rest.put().uri("/api/run-queue/settings")
                .body(Map.of("ceilingPercent", 140)).retrieve().toEntity(Map.class);
        assertThat(ceiling.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
    }

    @Test
    @DisplayName("waiting items reorder among themselves, and settled ones clear")
    void reorderAndClear() {
        UUID first = task("first", "First");
        UUID second = task("second", "Second");
        enqueue(first);
        enqueue(second);

        String secondItem = item(1).get("id").toString();
        Map<?, ?> moved = put("/api/run-queue/items/" + secondItem + "/position", Map.of("index", 0));

        assertThat(((Map<?, ?>) ((List<?>) moved.get("items")).getFirst()).get("taskId")).isEqualTo(second.toString());

        jdbc.update("UPDATE run_queue_item SET state = 'FINISHED' WHERE task_id = ?", first);
        Map<?, ?> cleared = post("/api/run-queue/clear", Map.of());
        assertThat((List<?>) cleared.get("items")).hasSize(1);
    }

    // ---------------------------------------------------------------- helpers

    private void usageAt(double sessionPercent) {
        List<Limit> limits = List.of(
                new Limit("session", "Session", sessionPercent, Severity.NORMAL, SESSION_RESET),
                new Limit("weekly_all", "Weekly · all models", 5, Severity.NORMAL, SESSION_RESET.plus(Duration.ofDays(3))));
        ClaudeUsageView view = ClaudeUsageView.ok(limits, Instant.now());
        when(usage.current()).thenReturn(view);
        when(usage.refresh()).thenReturn(view);
    }

    /** A step as the console leaves it once promoted: open, on the checklist a session reads. */
    private void openStep(UUID taskId, String title) {
        UUID stepId = steps.add(taskId, title, null).id();
        steps.edit(stepId, null, null, null, false);
    }

    private UUID task(String label, String title) {
        return UUID.fromString(id(post("/api/tasks", Map.of(
                "label", label, "title", title, "status", "IN_PROGRESS", "projectId", projectId))));
    }

    private void enqueue(UUID taskId) {
        post("/api/run-queue/items", Map.of("taskId", taskId.toString()));
    }

    private Map<?, ?> queue() {
        return rest.get().uri("/api/run-queue").retrieve().body(Map.class);
    }

    private Map<?, ?> item(int index) {
        return (Map<?, ?>) ((List<?>) queue().get("items")).get(index);
    }

    private String itemState(int index) {
        return String.valueOf(item(index).get("state"));
    }

    private String terminalOn(UUID taskId) {
        List<?> live = rest.get().uri("/api/terminals").retrieve().body(List.class);
        return live.stream()
                .map(row -> (Map<?, ?>) row)
                .filter(row -> taskId.toString().equals(row.get("taskId")))
                .map(row -> row.get("id").toString())
                .findFirst()
                .orElse(null);
    }

    private void closeLiveTerminals() {
        List<?> live = rest.get().uri("/api/terminals").retrieve().body(List.class);
        for (Object row : live) {
            rest.delete().uri("/api/terminals/" + ((Map<?, ?>) row).get("id")).retrieve().toBodilessEntity();
        }
    }

    private Map<?, ?> post(String path, Object body) {
        return rest.post().uri(path).body(body).retrieve().body(Map.class);
    }

    private Map<?, ?> put(String path, Map<String, ?> body) {
        return rest.put().uri(path).body(new HashMap<>(body)).retrieve().body(Map.class);
    }

    private static String id(Map<?, ?> body) {
        return String.valueOf(body.get("id"));
    }

    private static void await(BooleanSupplier condition) {
        Instant deadline = Instant.now().plus(Duration.ofSeconds(15));
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
        throw new AssertionError("condition not met within 15s");
    }

    private static Path writeStub() {
        try {
            Path stub = Files.createTempFile("fake-claude-queue", ".sh");
            Files.writeString(stub, STUB);
            assertThat(stub.toFile().setExecutable(true)).isTrue();
            return stub;
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }
}
