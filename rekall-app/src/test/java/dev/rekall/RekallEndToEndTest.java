package dev.rekall;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.test.web.server.LocalServerPort;
import org.springframework.http.HttpStatus;
import org.springframework.http.ProblemDetail;
import org.springframework.http.ResponseEntity;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.web.client.RestClient;

import java.io.ByteArrayInputStream;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The whole application, from entering data through the UI's API to answering with it over MCP.
 */
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class RekallEndToEndTest {

    @Autowired
    private JdbcTemplate jdbc;

    @LocalServerPort
    private int port;

    private RestClient rest;

    @BeforeEach
    void setUpClient() {
        // Status handling off so tests assert on codes, not thrown exceptions.
        rest = RestClient.builder()
                .baseUrl("http://localhost:" + port)
                .defaultStatusHandler(status -> true, (request, response) -> { })
                .build();
    }

    @BeforeEach
    void resetDatabase() {
        jdbc.execute("DELETE FROM task_step");
        jdbc.execute("DELETE FROM time_entry");
        jdbc.execute("DELETE FROM wrapup");
        jdbc.execute("DELETE FROM document_task");
        jdbc.execute("DELETE FROM document");
        jdbc.execute("DELETE FROM task");
        jdbc.execute("DELETE FROM project");
        jdbc.execute("DELETE FROM company");
    }

    @Test
    @DisplayName("one anchored call brings back the task, its project and every note on it")
    void loadsTheWholeWorkingContext() {
        String acme = aCompany("Acme");
        String projectId = id(post("/api/projects", Map.of(
                "label", "vega", "title", "Vega Platform", "status", "ACTIVE",
                "description", "Progetto Vega", "companyId", acme)));
        aProject(acme, "beacon", "PAUSED");
        String taskId = id(post("/api/tasks", Map.of(
                "label", "report-builder-main-workflow", "title", "Report builder, main workflow",
                "status", "IN_PROGRESS", "projectId", projectId)));

        post("/api/documents", Map.of(
                "title", "CONTEXT.md", "kind", "context", "taskIds", List.of(taskId),
                "bodyMarkdown", "# Contesto\n\nIl workflow parte da POST /api/v1/pipelines."));
        post("/api/documents", Map.of(
                "title", "kmaster14.md", "kind", "notes", "taskIds", List.of(taskId),
                "bodyMarkdown", "Cluster kmaster14, accesso via bastion."));

        String context = callTool("rekall_context", Map.of(
                "anchors", "project:vega task:report-builder-main-workflow"));

        assertThat(context)
                .as("the heading is what a person calls it")
                .contains("Project: Vega Platform")
                .contains("Task: Report builder, main workflow")
                .as("and the anchor line is what has to be typed to load it again")
                .contains("`project:vega`")
                .contains("`task:report-builder-main-workflow`")
                .as("the task's own notes arrive in full")
                .contains("POST /api/v1/pipelines")
                .contains("Cluster kmaster14, accesso via bastion.")
                .doesNotContain("beacon");

        assertThat(context.split("Project: Vega Platform", -1)).as("no record is rendered twice").hasSize(2);
    }

    @Test
    @DisplayName("a task's description arrives as a document, not as a bullet")
    void theDescriptionIsHandedOverWhole() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String description = """
                ## Cosa deve fare

                Il report builder genera il report settimanale.

                ## Fuori scope

                Il confronto fra settimane diverse.""";
        post("/api/tasks", Map.of(
                "label", "report-builder", "title", "Report builder", "status", "IN_PROGRESS",
                "description", description, "projectId", projectId));

        String context = callTool("rekall_context", Map.of("anchors", "task:report-builder"));

        assertThat(context)
                .as("in a tag, beside the wrapup and the notes")
                .contains("<description>")
                .contains("</description>")
                .as("whole, with the structure the brief was written with")
                .contains("## Fuori scope")
                .contains("Il confronto fra settimane diverse.")
                .as("and never flattened into the field list")
                .doesNotContain("- `description`");
    }

    @Test
    @DisplayName("a task's standing wrapup directive rides along with its context, and leaves when the toggle does")
    void theStandingWrapupDirectiveIsHandedOver() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = id(post("/api/tasks", Map.of(
                "label", "report-builder", "title", "Report builder", "status", "IN_PROGRESS",
                "autoWrapup", true, "wrapupDirective", "solo il modulo di export", "projectId", projectId)));

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .contains("- `wrapup`:")
                .contains("solo il modulo di export");

        rest.put().uri("/api/tasks/" + taskId)
                .body(Map.of("label", "report-builder", "title", "Report builder", "status", "IN_PROGRESS",
                        "autoWrapup", false, "wrapupDirective", "solo il modulo di export", "projectId", projectId))
                .retrieve().toEntity(Map.class);

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .doesNotContain("- `wrapup`:")
                .doesNotContain("solo il modulo di export");
    }

    @Test
    @DisplayName("the title can be rewritten and the anchor still loads the record")
    void titleChangesLeaveTheAnchorAlone() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = id(post("/api/tasks", Map.of(
                "label", "report-builder", "title", "Validator", "status", "TODO", "projectId", projectId)));

        rest.put().uri("/api/tasks/" + taskId)
                .body(Map.of("label", "report-builder", "title", "Report builder, main workflow",
                        "status", "IN_PROGRESS", "projectId", projectId))
                .retrieve().toEntity(Map.class);

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .contains("Task: Report builder, main workflow")
                .contains("IN_PROGRESS");
    }

    @Test
    @DisplayName("a label typed as a sentence is stored as the slug the anchor can carry")
    void labelsAreNormalised() {
        String acme = aCompany("Acme");
        ResponseEntity<Map> project = post("/api/projects", Map.of(
                "label", "  Vega Platform ", "title", "Vega Platform", "status", "ACTIVE",
                "companyId", acme));

        assertThat(project.getBody()).containsEntry("label", "vega-platform");
        assertThat(project.getBody()).containsEntry("anchor", "project:vega-platform");

        assertThat(callTool("rekall_context", Map.of("anchors", "project:vega-platform")))
                .contains("Project: Vega Platform");
    }

    @Test
    @DisplayName("a label with nothing usable left in it is refused as a bad request")
    void anEmptyLabelIsRejected() {
        String acme = aCompany("Acme");

        assertThat(post("/api/projects", Map.of(
                        "label", "///", "title", "Nowhere", "status", "ACTIVE", "companyId", acme))
                        .getStatusCode())
                .isEqualTo(HttpStatus.BAD_REQUEST);
    }

    @Test
    @DisplayName("two tasks on one project cannot share a label, and the refusal says why")
    void labelsAreUniqueWithinTheirParent() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        post("/api/tasks", Map.of("label", "setup", "title", "Setup", "status", "TODO", "projectId", projectId));

        ResponseEntity<Map> second = post("/api/tasks", Map.of(
                "label", "Setup", "title", "Setup again", "status", "TODO", "projectId", projectId));

        assertThat(second.getStatusCode()).isEqualTo(HttpStatus.CONFLICT);
        assertThat(String.valueOf(second.getBody().get("detail")))
                .as("the message is the one a person can act on, not the database's")
                .contains("already uses that label");
    }

    @Test
    @DisplayName("changing a label changes the anchor, and the old one stops resolving")
    void labelChangesMoveTheAnchor() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");

        rest.put().uri("/api/projects/" + projectId)
                .body(Map.of("label", "vega-2", "title", "Vega", "status", "ACTIVE", "companyId", acme))
                .retrieve().toEntity(Map.class);

        assertThat(callTool("rekall_context", Map.of("anchors", "project:vega-2"))).contains("Project: Vega");
        assertThat(callTool("rekall_context", Map.of("anchors", "project:vega")))
                .contains("No project matches 'vega'");
    }

    @Test
    @DisplayName("a project's folder reaches its tasks, and a blank one clears it")
    @SuppressWarnings("unchecked")
    void theProjectFolderReachesItsTasks() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        Map<String, Object> saved = updateProjectFolder(projectId, acme, "/Users/someone/Projects/vega");

        assertThat(saved.get("repoFolder")).isEqualTo("/Users/someone/Projects/vega");
        Map<String, Object> task = rest.get().uri("/api/tasks/" + taskId)
                .retrieve().toEntity(Map.class).getBody();
        assertThat(task.get("projectRepoFolder")).isEqualTo("/Users/someone/Projects/vega");

        assertThat(updateProjectFolder(projectId, acme, "   ").get("repoFolder")).isNull();
    }

    @SuppressWarnings("unchecked")
    private Map<String, Object> updateProjectFolder(String projectId, String companyId, String folder) {
        return rest.put().uri("/api/projects/" + projectId)
                .body(Map.of("label", "vega", "title", "Vega", "status", "ACTIVE",
                        "companyId", companyId, "repoFolder", folder))
                .retrieve().toEntity(Map.class).getBody();
    }

    @Test
    @DisplayName("a note attached to several tasks arrives with each of them")
    void oneNoteServesManyTasks() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String validator = aTask(projectId, "report-builder");
        String retry = aTask(projectId, "retry-policy");

        ResponseEntity<Map> shared = post("/api/documents", Map.of(
                "title", "kmaster14.md", "kind", "notes",
                "taskIds", List.of(validator, retry),
                "bodyMarkdown", "Accesso via bastion."));

        assertThat((List<?>) shared.getBody().get("tasks"))
                .as("the response names every task the note is on")
                .hasSize(2);

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .contains("Accesso via bastion.");
        assertThat(callTool("rekall_context", Map.of("anchors", "task:retry-policy")))
                .contains("Accesso via bastion.");

        // Editing through one task is visible from the other: one row, not two.
        String documentId = String.valueOf(shared.getBody().get("id"));
        rest.put().uri("/api/documents/" + documentId)
                .body(Map.of("title", "kmaster14.md", "kind", "notes",
                        "taskIds", List.of(validator, retry),
                        "bodyMarkdown", "Accesso via bastion, il certificato scade il 14."))
                .retrieve().toEntity(Map.class);

        assertThat(callTool("rekall_context", Map.of("anchors", "task:retry-policy")))
                .contains("il certificato scade il 14");
    }

    @Test
    @DisplayName("detaching a note from one task leaves it on the others")
    void detachingKeepsTheNoteAlive() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String first = aTask(projectId, "a");
        String second = aTask(projectId, "b");
        String documentId = id(post("/api/documents", Map.of(
                "title", "shared.md", "kind", "notes", "taskIds", List.of(first, second),
                "bodyMarkdown", "condivisa")));

        rest.put().uri("/api/documents/" + documentId)
                .body(Map.of("title", "shared.md", "kind", "notes",
                        "taskIds", List.of(second), "bodyMarkdown", "condivisa"))
                .retrieve().toEntity(Map.class);

        assertThat(documentsOn(first)).as("gone from the task it was detached from").isEmpty();
        assertThat(documentsOn(second)).as("still on the other one").hasSize(1);
    }

    @Test
    @DisplayName("a note has to be on at least one task, because nothing could reach it otherwise")
    void aNoteNeedsATask() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        aTask(projectId, "a");

        assertThat(post("/api/documents", Map.of(
                        "title", "x", "kind", "notes", "taskIds", List.of())).getStatusCode())
                .isEqualTo(HttpStatus.CONFLICT);
    }

    @Test
    @DisplayName("deleting a task keeps the notes that other tasks still use")
    void deletingATaskSweepsOnlyWhatIsOrphaned() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String doomed = aTask(projectId, "doomed");
        String survivor = aTask(projectId, "survivor");

        post("/api/documents", Map.of("title", "only-here.md", "kind", "notes",
                "taskIds", List.of(doomed), "bodyMarkdown", "sola"));
        post("/api/documents", Map.of("title", "shared.md", "kind", "notes",
                "taskIds", List.of(doomed, survivor), "bodyMarkdown", "condivisa"));

        rest.delete().uri("/api/tasks/" + doomed).retrieve().toEntity(Void.class);

        assertThat(documentsOn(survivor)).hasSize(1);
        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM document", Integer.class))
                .as("the note nothing pointed at is gone, the shared one is not")
                .isEqualTo(1);
    }

    @Test
    @DisplayName("deleting a project takes its tasks and sweeps the notes left on nothing")
    void deletingAProjectCascades() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String task = aTask(projectId, "report-builder");
        post("/api/documents", Map.of("title", "n.md", "kind", "notes",
                "taskIds", List.of(task), "bodyMarkdown", "x"));

        rest.delete().uri("/api/projects/" + projectId).retrieve().toEntity(Void.class);

        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM task", Integer.class)).isZero();
        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM document", Integer.class)).isZero();
    }

    @Test
    @DisplayName("a single project anchor lists its tasks as anchors, without their bodies")
    void projectAnchorListsTasks() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder-main-workflow");
        post("/api/documents", Map.of(
                "title", "CONTEXT.md", "kind", "context", "taskIds", List.of(taskId),
                "bodyMarkdown", "segreto"));

        String context = callTool("rekall_context", Map.of("anchors", "project:vega"));

        assertThat(context)
                .contains("`task:report-builder-main-workflow`")
                .as("an inverse listing carries anchors only, never the referenced bodies")
                .doesNotContain("segreto");
    }

    @Test
    @DisplayName("a bare term resolves when it is unambiguous and reports the candidates when it is not")
    void positionalAnchors() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        aTask(projectId, "vega");

        assertThat(callTool("rekall_context", Map.of("anchors", "report-builder")))
                .contains("Nothing matches 'report-builder'");
        assertThat(callTool("rekall_context", Map.of("anchors", "vega")))
                .as("a project and a task both labelled vega must stop the tool, not be guessed at")
                .contains("matches 2 records")
                .contains("Qualify it as `entity:value`");
    }

    @Test
    @DisplayName("a task label that two projects share is disambiguated by the project anchor")
    void taskLabelSharedAcrossProjects() {
        String acme = aCompany("Acme");
        String vega = aProject(acme, "vega", "ACTIVE");
        String beacon = aProject(acme, "beacon", "ACTIVE");
        post("/api/tasks", Map.of("label", "setup", "title", "Setup", "status", "TODO", "projectId", vega));
        post("/api/tasks", Map.of("label", "setup", "title", "Setup", "status", "DONE", "projectId", beacon));

        assertThat(callTool("rekall_context", Map.of("anchors", "task:setup")))
                .contains("matches 2 records");
        assertThat(callTool("rekall_context", Map.of("anchors", "project:beacon task:setup")))
                .contains("Task: Setup")
                .contains("DONE");
    }

    // Asserted as an exact list so adding a read or write tool breaks a test.
    @Test
    @DisplayName("the MCP endpoint exposes one way to read and two writes")
    void toolsList() {
        List<?> tools = (List<?>) ((Map<?, ?>) rpc("tools/list", Map.of()).get("result")).get("tools");

        assertThat(tools.stream().map(tool -> String.valueOf(((Map<?, ?>) tool).get("name"))))
                .containsExactlyInAnyOrder("rekall_context", "rekall_wrapup", "rekall_step");
    }

    // --- Wrapup

    @Test
    @DisplayName("a wrapup written over MCP arrives with the next context load")
    void wrapupIsWrittenAndComesBack() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        aTask(projectId, "report-builder");

        String written = callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder",
                "body", "## Stato\n\nIl builder legge da POST /api/v1/pipelines."));

        assertThat(written)
                .contains("Wrapup written for")
                .contains("project:vega task:report-builder");

        assertThat(callTool("rekall_context", Map.of("anchors", "project:vega task:report-builder")))
                .as("it comes back inside its own tag, not as one of the notes")
                .contains("<wrapup written-by=\"CLAUDE\"")
                .contains("Il builder legge da POST /api/v1/pipelines.");
    }

    @Test
    @DisplayName("writing a second wrapup replaces the first rather than adding one")
    void wrapupIsReplacedNotAppended() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Il primo stato."));
        String second = callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Lo stato corrente."));

        assertThat(second).contains("Wrapup replaced for");
        assertThat(jdbc.queryForObject(
                        "SELECT COUNT(*) FROM wrapup WHERE task_id = ?", Integer.class, UUID.fromString(taskId)))
                .isEqualTo(1);

        String context = callTool("rekall_context", Map.of("anchors", "task:report-builder"));
        assertThat(context).contains("Lo stato corrente.").doesNotContain("Il primo stato.");
    }

    @Test
    @DisplayName("replacing a hand-written wrapup says so, and the author follows the last writer")
    void wrapupReportsWhoseWordsItReplaced() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        ResponseEntity<Map> byHand = rest.put().uri("/api/tasks/" + taskId + "/wrapup")
                .body(Map.of("bodyMarkdown", "Scritto a mano."))
                .retrieve().toEntity(Map.class);

        assertThat(byHand.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(byHand.getBody()).containsEntry("writtenBy", "HAND");
        assertThat(byHand.getBody()).containsEntry("anchor", "project:vega task:report-builder");

        assertThat(callTool("rekall_wrapup", Map.of(
                        "anchors", "project:vega task:report-builder", "body", "Riscritto da Claude.")))
                .contains("had been edited by hand");

        assertThat(rest.get().uri("/api/tasks/" + taskId + "/wrapup")
                        .retrieve().toEntity(Map.class).getBody())
                .containsEntry("writtenBy", "CLAUDE");
    }

    @Test
    @DisplayName("a wrapup refuses any anchor that does not name exactly one task")
    void wrapupNeedsExactlyOneTask() {
        String acme = aCompany("Acme");
        String vega = aProject(acme, "vega", "ACTIVE");
        String beacon = aProject(acme, "beacon", "ACTIVE");
        aTask(vega, "setup");
        aTask(beacon, "setup");

        assertThat(callTool("rekall_wrapup", Map.of("anchors", "project:vega", "body", "x")))
                .as("a project names forty tasks and none of them is the answer")
                .contains("This write belongs to exactly one task");

        assertThat(callTool("rekall_wrapup", Map.of("anchors", "task:setup", "body", "x")))
                .contains("matches 2 records")
                .contains("Qualify it with `project:");

        assertThat(callTool("rekall_wrapup", Map.of("anchors", "task:nowhere", "body", "x")))
                .contains("No task matches 'nowhere'");

        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM wrapup", Integer.class))
                .as("nothing was written on any of those")
                .isZero();
    }

    @Test
    @DisplayName("a wrapup that has grown into a log is refused, and told why")
    void wrapupIsCapped() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        aTask(projectId, "report-builder");

        assertThat(callTool("rekall_wrapup", Map.of(
                        "anchors", "project:vega task:report-builder", "body", "x".repeat(20_001))))
                .contains("capped at 20000 characters")
                .contains("not how it got there");

        assertThat(callTool("rekall_wrapup", Map.of(
                        "anchors", "project:vega task:report-builder", "body", "   ")))
                .contains("'body' is required");
        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM wrapup", Integer.class)).isZero();
    }

    @Test
    @DisplayName("deleting a task takes its wrapup, and deleting a wrapup leaves the task")
    void wrapupCascadesWithItsTask() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Lo stato."));

        rest.delete().uri("/api/tasks/" + taskId + "/wrapup").retrieve().toEntity(Void.class);

        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM wrapup", Integer.class)).isZero();
        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM task", Integer.class))
                .as("the task and its notes are untouched")
                .isEqualTo(1);

        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Di nuovo."));
        rest.delete().uri("/api/tasks/" + taskId).retrieve().toEntity(Void.class);

        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM wrapup", Integer.class))
                .as("and the row goes with the task it describes")
                .isZero();
    }

    @Test
    @DisplayName("a task reports whether it has a wrapup, without carrying the body")
    void taskRowsReportWhetherTheyHaveAWrapup() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        aTask(projectId, "report-builder");
        aTask(projectId, "retry-policy");
        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Lo stato."));

        List<?> tasks = rest.get().uri("/api/tasks").retrieve().toEntity(List.class).getBody();

        assertThat(tasks).hasSize(2);
        assertThat(tasks.stream().map(task ->
                        ((Map<?, ?>) task).get("label") + "=" + ((Map<?, ?>) task).get("hasWrapup")))
                .containsExactlyInAnyOrder("report-builder=true", "retry-policy=false");
    }

    // --- Time entries

    @Test
    @DisplayName("starting a timer opens a session, and stopping closes it")
    void startingAndStoppingATimer() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        Map<?, ?> started = post("/api/tasks/" + taskId + "/time-entries/start", Map.of()).getBody();
        assertThat(started.get("taskId")).isEqualTo(taskId);
        assertThat(started.get("stoppedAt")).isNull();

        Map<?, ?> stopped = post("/api/tasks/" + taskId + "/time-entries/stop", Map.of()).getBody();
        assertThat(stopped.get("id")).isEqualTo(started.get("id"));
        assertThat(stopped.get("stoppedAt")).isNotNull();
    }

    @Test
    @DisplayName("starting a second task's timer leaves the first one running")
    void startingASecondTaskDoesNotStopTheFirst() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskA = aTask(projectId, "task-a");
        String taskB = aTask(projectId, "task-b");

        Map<?, ?> firstStarted = post("/api/tasks/" + taskA + "/time-entries/start", Map.of()).getBody();
        Map<?, ?> secondStarted = post("/api/tasks/" + taskB + "/time-entries/start", Map.of()).getBody();

        assertThat(secondStarted.get("taskId")).isEqualTo(taskB);
        assertThat(secondStarted.get("stoppedAt")).isNull();

        List<?> entries = rest.get().uri("/api/time-entries").retrieve().toEntity(List.class).getBody();
        Map<?, ?> stillRunning = entries.stream()
                .map(entry -> (Map<?, ?>) entry)
                .filter(entry -> entry.get("id").equals(firstStarted.get("id")))
                .findFirst()
                .orElseThrow();
        assertThat(stillRunning.get("stoppedAt")).isNull();
    }

    @Test
    @DisplayName("starting a timer that is already running on this task is a no-op")
    void startingWhatIsAlreadyRunningIsANoOp() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        Map<?, ?> first = post("/api/tasks/" + taskId + "/time-entries/start", Map.of()).getBody();
        Map<?, ?> again = post("/api/tasks/" + taskId + "/time-entries/start", Map.of()).getBody();

        assertThat(again.get("id")).isEqualTo(first.get("id"));
        assertThat(jdbc.queryForObject(
                        "SELECT COUNT(*) FROM time_entry WHERE task_id = ?", Integer.class, UUID.fromString(taskId)))
                .isEqualTo(1);
    }

    @Test
    @DisplayName("stopping a task that is not being tracked is refused")
    void stoppingWithNothingRunningIsRefused() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        assertThat(post("/api/tasks/" + taskId + "/time-entries/stop", Map.of()).getStatusCode())
                .isEqualTo(HttpStatus.BAD_REQUEST);
    }

    @Test
    @DisplayName("moving a task to DONE stops the session running on it")
    void movingATaskToDoneStopsItsTimer() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        Map<?, ?> started = post("/api/tasks/" + taskId + "/time-entries/start", Map.of()).getBody();

        rest.put().uri("/api/tasks/" + taskId)
                .body(Map.of("label", "report-builder", "title", "report-builder",
                        "status", "DONE", "projectId", projectId))
                .retrieve().toEntity(Map.class);

        List<?> entries = rest.get().uri("/api/time-entries").retrieve().toEntity(List.class).getBody();
        Map<?, ?> session = entries.stream()
                .map(entry -> (Map<?, ?>) entry)
                .filter(entry -> entry.get("id").equals(started.get("id")))
                .findFirst()
                .orElseThrow();
        assertThat(session.get("stoppedAt")).isNotNull();
    }

    @Test
    @DisplayName("moving a task to a status other than DONE leaves its timer running")
    void movingATaskToAnotherStatusLeavesItsTimerRunning() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        post("/api/tasks/" + taskId + "/time-entries/start", Map.of());

        rest.put().uri("/api/tasks/" + taskId)
                .body(Map.of("label", "report-builder", "title", "report-builder",
                        "status", "BLOCKED", "projectId", projectId))
                .retrieve().toEntity(Map.class);

        List<?> entries = rest.get().uri("/api/time-entries").retrieve().toEntity(List.class).getBody();
        assertThat(entries.stream().map(entry -> ((Map<?, ?>) entry).get("stoppedAt")))
                .containsOnlyNulls();
    }

    @Test
    @DisplayName("moving a task to DONE with nothing running is fine")
    void movingATaskToDoneWithNothingRunningIsFine() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        assertThat(rest.put().uri("/api/tasks/" + taskId)
                        .body(Map.of("label", "report-builder", "title", "report-builder",
                                "status", "DONE", "projectId", projectId))
                        .retrieve().toEntity(Map.class).getStatusCode())
                .isEqualTo(HttpStatus.OK);
    }

    @Test
    @DisplayName("a session can be corrected by hand, within the rules that keep it sane")
    void sessionsAreCorrectedByHand() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        Map<?, ?> started = post("/api/tasks/" + taskId + "/time-entries/start", Map.of()).getBody();
        Map<?, ?> stopped = post("/api/tasks/" + taskId + "/time-entries/stop", Map.of()).getBody();
        String entryId = String.valueOf(stopped.get("id"));

        assertThat(rest.patch().uri("/api/time-entries/" + entryId)
                        .body(Map.of("startedAt", stopped.get("stoppedAt"), "stoppedAt", started.get("startedAt")))
                        .retrieve().toEntity(Map.class).getStatusCode())
                .as("a session has to end after it starts")
                .isEqualTo(HttpStatus.BAD_REQUEST);

        assertThat(rest.patch().uri("/api/time-entries/" + entryId)
                        .body(Map.of("startedAt", started.get("startedAt")))
                        .retrieve().toEntity(Map.class).getStatusCode())
                .as("a finished session cannot be reopened")
                .isEqualTo(HttpStatus.BAD_REQUEST);

        ResponseEntity<Map> corrected = rest.patch().uri("/api/time-entries/" + entryId)
                .body(Map.of("startedAt", started.get("startedAt"), "stoppedAt", stopped.get("stoppedAt")))
                .retrieve().toEntity(Map.class);
        assertThat(corrected.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(corrected.getBody()).containsEntry("startedAt", started.get("startedAt"));
    }

    @Test
    @DisplayName("deleting a session removes only that one")
    void deletingASession() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        post("/api/tasks/" + taskId + "/time-entries/start", Map.of());
        String first = String.valueOf(post("/api/tasks/" + taskId + "/time-entries/stop", Map.of())
                .getBody().get("id"));
        post("/api/tasks/" + taskId + "/time-entries/start", Map.of());
        String second = String.valueOf(post("/api/tasks/" + taskId + "/time-entries/stop", Map.of())
                .getBody().get("id"));

        assertThat(rest.delete().uri("/api/time-entries/" + first).retrieve().toEntity(Void.class)
                        .getStatusCode())
                .isEqualTo(HttpStatus.NO_CONTENT);

        assertThat(jdbc.queryForObject(
                        "SELECT COUNT(*) FROM time_entry WHERE task_id = ?", Integer.class, UUID.fromString(taskId)))
                .as("the other session on the same task survives")
                .isEqualTo(1);
        List<?> remaining = rest.get().uri("/api/time-entries").retrieve().toEntity(List.class).getBody();
        assertThat(remaining.stream().map(entry -> String.valueOf(((Map<?, ?>) entry).get("id"))))
                .containsExactly(second);
    }

    @Test
    @DisplayName("deleting a task takes its time entries with it")
    void timeEntriesCascadeWithTheirTask() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        post("/api/tasks/" + taskId + "/time-entries/start", Map.of());
        post("/api/tasks/" + taskId + "/time-entries/stop", Map.of());

        rest.delete().uri("/api/tasks/" + taskId).retrieve().toEntity(Void.class);

        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM time_entry", Integer.class)).isZero();
    }

    // --- 2026-07-28 era over real HTTP: the only place that proves Spring binds the mirrored headers.

    @Test
    @DisplayName("a stateless-era call is served with no handshake before it")
    void statelessEraNeedsNoInitialize() {
        String acme = aCompany("Acme");
        aProject(acme, "vega", "ACTIVE");

        ResponseEntity<Map> response = modernRpc("tools/call", "rekall_context",
                Map.of("name", "rekall_context", "arguments", Map.of("anchors", "project:vega")));

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        Map<?, ?> result = (Map<?, ?>) response.getBody().get("result");
        assertThat(result.get("isError")).isEqualTo(false);
        assertThat(String.valueOf(((Map<?, ?>) ((List<?>) result.get("content")).getFirst()).get("text")))
                .contains("project:vega");
    }

    @Test
    @DisplayName("a stateless-era call whose headers disagree with its body is refused")
    void statelessEraRefusesAHeaderMismatch() {
        ResponseEntity<Map> response = modernRpc("tools/call", "something_else",
                Map.of("name", "rekall_context", "arguments", Map.of("anchors", "project:vega")));

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
        assertThat(((Map<?, ?>) response.getBody().get("error")).get("code")).isEqualTo(-32020);
    }

    @Test
    @DisplayName("server/discover names the revisions this server speaks")
    void discoverNamesTheSupportedRevisions() {
        ResponseEntity<Map> response = modernRpc("server/discover", null, Map.of());

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        Map<?, ?> result = (Map<?, ?>) response.getBody().get("result");
        @SuppressWarnings("unchecked")
        List<String> versions = (List<String>) result.get("supportedVersions");
        assertThat(versions).contains("2026-07-28");
    }

    // Regression: returning the entity before flush left updatedAt null and the frontend rejected the create.
    @Test
    @DisplayName("a create answers with the timestamps already written, not with nulls")
    void createResponsesCarryTheirTimestamps() {
        String acme = aCompany("Acme");
        ResponseEntity<Map> project = post("/api/projects", Map.of(
                "label", "vega", "title", "Vega", "status", "ACTIVE", "companyId", acme));
        assertThat(project.getBody()).doesNotContainEntry("updatedAt", null);
        String projectId = id(project);

        ResponseEntity<Map> task = post("/api/tasks", Map.of(
                "label", "report-builder", "title", "Report builder", "status", "TODO",
                "projectId", projectId));
        assertThat(task.getBody()).doesNotContainEntry("updatedAt", null);

        assertThat(post("/api/documents", Map.of(
                        "title", "CONTEXT.md", "kind", "context",
                        "taskIds", List.of(id(task)), "bodyMarkdown", "x"))
                        .getBody())
                .doesNotContainEntry("updatedAt", null);
    }

    @Test
    @DisplayName("a status outside the enum is the caller's mistake, so it is a 400 and not a 500")
    void unparseableBodyIsABadRequest() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");

        assertThat(post("/api/tasks", Map.of(
                        "label", "t", "title", "T", "status", "OPEN", "projectId", projectId))
                        .getStatusCode())
                .isEqualTo(HttpStatus.BAD_REQUEST);
    }

    @Test
    @DisplayName("both a label and a title are required")
    void bothNamesAreRequired() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");

        assertThat(post("/api/tasks", Map.of("label", "t", "status", "TODO", "projectId", projectId))
                        .getStatusCode())
                .isEqualTo(HttpStatus.BAD_REQUEST);
        assertThat(post("/api/tasks", Map.of("title", "T", "status", "TODO", "projectId", projectId))
                        .getStatusCode())
                .isEqualTo(HttpStatus.BAD_REQUEST);
    }

    @Test
    @DisplayName("every task, unscoped, comes back grouped by project and then by label")
    void unscopedTaskListIsOrdered() {
        String acme = aCompany("Acme");
        String vega = aProject(acme, "vega", "ACTIVE");
        String beacon = aProject(acme, "beacon", "ACTIVE");
        aTask(vega, "setup");
        aTask(vega, "report-builder");
        aTask(beacon, "wiring");

        List<?> tasks = rest.get().uri("/api/tasks").retrieve().toEntity(List.class).getBody();

        assertThat(tasks).hasSize(3);
        assertThat(tasks.stream().map(task ->
                        ((Map<?, ?>) task).get("projectLabel") + "/" + ((Map<?, ?>) task).get("label")))
                .containsExactly("beacon/wiring", "vega/report-builder", "vega/setup");
    }

    // Regression: a stale forwarding list left current routes answering 404 on a refresh.
    @Test
    @DisplayName("a refresh on any ui route serves the application, and an unknown api path still fails")
    void deepLinksReachTheFrontend() {
        for (String path : List.of("/", "/projects", "/projects/" + UUID.randomUUID(), "/tasks", "/report",
                "/tasks/" + UUID.randomUUID(), "/search", "/calendar")) {
            ResponseEntity<String> response = rest.get().uri(path).retrieve().toEntity(String.class);

            assertThat(response.getStatusCode()).as("GET %s", path).isEqualTo(HttpStatus.OK);
            assertThat(response.getBody()).as("GET %s serves the spa", path).contains("<div id=\"app\">");
        }

        assertThat(rest.get().uri("/api/nope").retrieve().toEntity(String.class).getStatusCode())
                .as("an unknown api path must not quietly answer with html")
                .isEqualTo(HttpStatus.NOT_FOUND);
    }

    @Test
    @DisplayName("nothing answers on the environment endpoints any more")
    void environmentsAreGone() {
        assertThat(rest.get().uri("/api/environments").retrieve().toEntity(String.class).getStatusCode())
                .isEqualTo(HttpStatus.NOT_FOUND);
        assertThat(jdbc.queryForList(
                        "SELECT table_name FROM information_schema.tables WHERE UPPER(table_name) = 'ENVIRONMENT'"))
                .as("the table is dropped, not merely unused")
                .isEmpty();
    }

    // --- Steps

    @Test
    @DisplayName("open steps reach Claude in full, done steps the wrapup covers by name alone")
    void openStepsCarryTheirDetailAndDoneStepsDoNot() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        String aggregate = aStep(taskId, "Aggregate the rows", "Somma per settimana, gruppo per progetto.");
        aStep(taskId, "Write the tests", "Un caso per settimana vuota e uno per settimana piena.");

        rest.patch().uri("/api/steps/" + aggregate).body(Map.of("done", true))
                .retrieve().toEntity(Map.class);
        // Written after the tick, so the finished step is accounted for.
        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Le righe sono aggregate."));

        String context = callTool("rekall_context", Map.of("anchors", "task:report-builder"));

        assertThat(context)
                .as("the shape of what is left is legible before the list is read")
                .contains("- `steps`: 1 of 2 done, 1 open")
                .contains("<steps done=\"1\" open=\"1\">")
                .as("both are named, in the order the work is meant to happen")
                .contains("- [x] Aggregate the rows")
                .contains("- [ ] Write the tests")
                .as("the open one carries what it has to do")
                .contains("Un caso per settimana vuota")
                .as("and the finished one does not")
                .doesNotContain("Somma per settimana");
    }

    @Test
    @DisplayName("a step finished after the last wrapup is marked, and gets its detail back")
    void stepsFinishedSinceTheWrapupAreMarked() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        String early = aStep(taskId, "Modello e migrazione", "Entita Report, changeset Liquibase.");
        String late = aStep(taskId, "Aggregazione delle righe", "Somma per settimana, gruppo per progetto.");
        aStep(taskId, "Scrivere i test", "Un caso per settimana vuota.");

        // First step finished and described; the wrapup knows about it.
        rest.patch().uri("/api/steps/" + early).body(Map.of("done", true)).retrieve().toEntity(Map.class);
        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Il modello esiste."));

        // Second ticked afterwards with no wrapup: the state a session opens on.
        rest.patch().uri("/api/steps/" + late).body(Map.of("done", true)).retrieve().toEntity(Map.class);

        String context = callTool("rekall_context", Map.of("anchors", "task:report-builder"));

        assertThat(context)
                .as("the count says how much the wrapup is behind by")
                .contains("finished-since-wrapup=\"1\"")
                .as("and the step itself is named as one the wrapup predates")
                .contains("- [x] Aggregazione delle righe  (finished since the wrapup was written)")
                .as("with its detail back, because nothing else here says what that piece was")
                .contains("Somma per settimana, gruppo per progetto.")
                .as("the step the wrapup already covers stays a title and nothing more")
                .contains("- [x] Modello e migrazione\n")
                .doesNotContain("changeset Liquibase");

        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder",
                "body", "Il modello esiste e le righe sono aggregate per settimana."));

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .as("nothing is behind any more, so nothing is marked")
                .doesNotContain("finished-since-wrapup")
                .doesNotContain("finished since the wrapup was written")
                .as("and the detail goes silent again")
                .doesNotContain("Somma per settimana, gruppo per progetto.");
    }

    @Test
    @DisplayName("with no wrapup yet, every finished step is marked and carries its detail")
    void withoutAWrapupEveryFinishedStepIsUnaccountedFor() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        String first = aStep(taskId, "Modello", "Entita Report.");
        String second = aStep(taskId, "Endpoint", "POST /api/v1/reports.");
        rest.patch().uri("/api/steps/" + first).body(Map.of("done", true)).retrieve().toEntity(Map.class);
        rest.patch().uri("/api/steps/" + second).body(Map.of("done", true)).retrieve().toEntity(Map.class);

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .contains("finished-since-wrapup=\"2\"")
                .contains("Entita Report.")
                .contains("POST /api/v1/reports.");
    }

    @Test
    @DisplayName("a task with no steps carries no steps block")
    void aTaskWithoutStepsSaysNothing() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        aTask(projectId, "report-builder");

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .doesNotContain("<steps")
                .doesNotContain("`steps`");
    }

    @Test
    @DisplayName("a draft step is the creation default, is kept off the session's checklist, and promotes to open")
    void aDraftStepIsHeldBackUntilPromoted() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        Map<?, ?> created = post("/api/tasks/" + taskId + "/steps",
                Map.of("title", "Aggregate the rows", "bodyMarkdown", "Somma per settimana."))
                .getBody();
        assertThat(created.get("state")).as("a step is born a draft").isEqualTo("DRAFT");
        String stepId = String.valueOf(created.get("id"));

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .as("a task that only holds drafts has no checklist block")
                .doesNotContain("<steps")
                .as("but the draft is counted, so the reader knows planning is under way")
                .contains("- `drafts`: 1 not yet promoted to the checklist")
                .as("and its detail never leaks to a session")
                .doesNotContain("Somma per settimana");

        assertThat(callTool("rekall_step", Map.of(
                "anchors", "project:vega task:report-builder", "step", "1", "state", "running")))
                .as("a session cannot start work on a draft")
                .contains("still a draft");

        rest.patch().uri("/api/steps/" + stepId).body(Map.of("draft", false))
                .retrieve().toEntity(Map.class);

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .as("once promoted it is an ordinary open step, detail and all")
                .contains("<steps done=\"0\" open=\"1\">")
                .contains("- [ ] Aggregate the rows")
                .contains("Somma per settimana.")
                .doesNotContain("`drafts`");
    }

    @Test
    @DisplayName("an open step returns to draft; a claimed one does not")
    void draftStateOnlyMovesWhileAStepIsUntouched() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        String stepId = aStep(taskId, "Aggregate the rows", null);

        Map<?, ?> backToDraft = rest.patch().uri("/api/steps/" + stepId)
                .body(Map.of("draft", true)).retrieve().toEntity(Map.class).getBody();
        assertThat(backToDraft.get("state")).isEqualTo("DRAFT");

        // Promote, claim it the way a session would, then try to send it back.
        rest.patch().uri("/api/steps/" + stepId).body(Map.of("draft", false))
                .retrieve().toEntity(Map.class);
        callTool("rekall_step", Map.of(
                "anchors", "project:vega task:report-builder", "step", "1", "state", "claimed"));

        ResponseEntity<Map> refused = rest.patch().uri("/api/steps/" + stepId)
                .body(Map.of("draft", true)).retrieve().toEntity(Map.class);
        assertThat(refused.getStatusCode().is4xxClientError()).isTrue();
    }

    @Test
    @DisplayName("steps are appended, reordered, and renumbered when one is removed")
    void stepsKeepADenseOrder() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        String first = aStep(taskId, "First", null);
        aStep(taskId, "Second", null);
        String third = aStep(taskId, "Third", null);

        assertThat(labelsOfSteps(taskId)).containsExactly("First", "Second", "Third");

        List<?> reordered = rest.post().uri("/api/steps/" + third + "/move")
                .body(Map.of("position", 0)).retrieve().toEntity(List.class).getBody();
        assertThat(reordered.stream().map(step -> String.valueOf(((Map<?, ?>) step).get("title"))))
                .as("the whole list comes back, because a move renumbers what it displaced")
                .containsExactly("Third", "First", "Second");

        rest.delete().uri("/api/steps/" + first).retrieve().toEntity(Void.class);

        assertThat(labelsOfSteps(taskId)).containsExactly("Third", "Second");
        assertThat(jdbc.queryForList(
                        "SELECT position FROM task_step WHERE task_id = ? ORDER BY position",
                        Integer.class, UUID.fromString(taskId)))
                .as("dense from zero, with no gap where the deleted one was")
                .containsExactly(0, 1);
    }

    @Test
    @DisplayName("moving a step past the end of the list puts it at the end")
    void movingPastTheEndClamps() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        String first = aStep(taskId, "First", null);
        aStep(taskId, "Second", null);

        rest.post().uri("/api/steps/" + first + "/move").body(Map.of("position", 99))
                .retrieve().toEntity(List.class);

        assertThat(labelsOfSteps(taskId)).containsExactly("Second", "First");
    }

    @Test
    @DisplayName("a step is ticked without carrying its title or its detail")
    void aStepIsTickedOnItsOwn() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        String stepId = aStep(taskId, "Write the tests", "Un caso per settimana vuota.");

        Map<?, ?> ticked = rest.patch().uri("/api/steps/" + stepId).body(Map.of("done", true))
                .retrieve().toEntity(Map.class).getBody();

        assertThat(ticked.get("done")).isEqualTo(true);
        assertThat(ticked.get("title")).isEqualTo("Write the tests");
        assertThat(ticked.get("bodyMarkdown")).isEqualTo("Un caso per settimana vuota.");
        assertThat(ticked.get("doneAt")).as("the flag and the moment are one fact").isNotNull();

        Map<?, ?> reopened = rest.patch().uri("/api/steps/" + stepId).body(Map.of("done", false))
                .retrieve().toEntity(Map.class).getBody();

        assertThat(reopened.get("done")).isEqualTo(false);
        assertThat(reopened.get("doneAt")).as("and it is cleared again when it reopens").isNull();
    }

    @Test
    @DisplayName("a task row reports how much of its checklist is done")
    void taskRowsReportTheirProgress() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        aTask(projectId, "retry-policy");
        String first = aStep(taskId, "First", null);
        aStep(taskId, "Second", null);
        rest.patch().uri("/api/steps/" + first).body(Map.of("done", true)).retrieve().toEntity(Map.class);

        List<?> tasks = rest.get().uri("/api/tasks").retrieve().toEntity(List.class).getBody();

        assertThat(tasks.stream().map(task -> {
                    Map<?, ?> row = (Map<?, ?>) task;
                    return row.get("label") + "=" + row.get("stepsDone") + "/" + row.get("stepCount");
                }))
                .containsExactlyInAnyOrder("report-builder=1/2", "retry-policy=0/0");
    }

    @Test
    @DisplayName("deleting a task takes its checklist with it")
    void stepsCascadeWithTheirTask() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        aStep(taskId, "First", null);
        aStep(taskId, "Second", null);

        rest.delete().uri("/api/tasks/" + taskId).retrieve().toEntity(Void.class);

        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM task_step", Integer.class)).isZero();
    }

    @Test
    @DisplayName("a step with no title, or with a detail the size of a document, is refused")
    void stepsAreCheckedBeforeTheyAreStored() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        Map<String, Object> untitled = new LinkedHashMap<>();
        untitled.put("title", "   ");
        untitled.put("bodyMarkdown", null);
        assertThat(rest.post().uri("/api/tasks/" + taskId + "/steps").body(untitled)
                        .retrieve().toEntity(Map.class).getStatusCode())
                .isEqualTo(HttpStatus.BAD_REQUEST);

        assertThat(rest.post().uri("/api/tasks/" + taskId + "/steps")
                        .body(Map.of("title", "Too much", "bodyMarkdown", "x".repeat(20_001)))
                        .retrieve().toEntity(ProblemDetail.class).getBody().getDetail())
                .contains("capped at 20000 characters")
                .contains("a task of its own");

        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM task_step", Integer.class)).isZero();
    }

    // --- Live steps

    @Test
    @DisplayName("marking a step running shows it as in progress on the next load")
    void aStepGoesRunning() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        aStep(taskId, "Aggregate the rows", "Somma per settimana.");
        aStep(taskId, "Write the tests", null);

        assertThat(callTool("rekall_step", Map.of(
                        "anchors", "project:vega task:report-builder", "step", "1", "state", "running")))
                .contains("is now `running`")
                .as("and it points at the step to pick up next")
                .contains("Next open step: 2 \"Write the tests\"");

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .contains("<steps done=\"0\" open=\"2\" running=\"1\">")
                .contains("- [ ] Aggregate the rows  (in progress)")
                .as("a running step is still work, so its detail is still on screen")
                .contains("Somma per settimana.");
    }

    @Test
    @DisplayName("a claimed step is finished work, waiting for the console to accept it")
    void aStepIsClaimed() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        String first = aStep(taskId, "Aggregate the rows", "Somma per settimana.");
        aStep(taskId, "Write the tests", null);

        callTool("rekall_step", Map.of(
                "anchors", "project:vega task:report-builder", "step", "1", "state", "running"));
        assertThat(callTool("rekall_step", Map.of(
                        "anchors", "project:vega task:report-builder", "step", "1", "state", "claimed")))
                .contains("is now `claimed`")
                .contains("Write the wrapup");
        // Written after the claim, so the wrapup accounts for it.
        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Le righe sono aggregate."));

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .contains("<steps done=\"1\" open=\"1\" awaiting-review=\"1\">")
                .contains("- [x] Aggregate the rows  (claimed, waiting for the console to accept it)");

        // The progress count is built on what a person accepted, so a claimed step is not done there.
        List<?> tasks = rest.get().uri("/api/tasks").retrieve().toEntity(List.class).getBody();
        assertThat(tasks.stream().map(task ->
                        ((Map<?, ?>) task).get("label") + "=" + ((Map<?, ?>) task).get("stepsDone")))
                .containsExactly("report-builder=0");

        rest.patch().uri("/api/steps/" + first).body(Map.of("done", true)).retrieve().toEntity(Map.class);
        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .contains("<steps done=\"1\" open=\"1\">")
                .doesNotContain("awaiting-review");
    }

    @Test
    @DisplayName("a session cannot mark a step done, only claimed")
    void aSessionCannotTickTheLastBox() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        aStep(taskId, "Aggregate the rows", null);

        assertThat(callTool("rekall_step", Map.of(
                        "anchors", "project:vega task:report-builder", "step", "1", "state", "done")))
                .as("said as `claimed`, and told why")
                .contains("Marked `claimed`, not done")
                .contains("cannot tick the last box");

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .as("claimed, not done: it is waiting for the console, not ticked")
                .contains("awaiting-review=\"1\"")
                .contains("(claimed, waiting for the console to accept it)");

        List<?> tasks = rest.get().uri("/api/tasks").retrieve().toEntity(List.class).getBody();
        assertThat(((Map<?, ?>) tasks.getFirst()).get("stepsDone")).isEqualTo(0);
    }

    @Test
    @DisplayName("a step reference that matches nothing is refused with the list")
    void anUnknownStepIsRefused() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        aStep(taskId, "Aggregate the rows", null);

        assertThat(callTool("rekall_step", Map.of(
                        "anchors", "project:vega task:report-builder", "step", "7", "state", "running")))
                .contains("There is no step 7");
        assertThat(callTool("rekall_step", Map.of(
                        "anchors", "project:vega task:report-builder", "step", "nope", "state", "running")))
                .contains("No step matches 'nope'")
                .contains("1. Aggregate the rows");
    }

    @Test
    @DisplayName("a step claimed before the wrapup stays covered when the console ticks it later")
    void aClaimedStepIsMeasuredFromWhenItWasClaimed() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        String first = aStep(taskId, "Aggregate the rows", "Somma per settimana.");
        aStep(taskId, "Write the tests", null);

        callTool("rekall_step", Map.of(
                "anchors", "project:vega task:report-builder", "step", "1", "state", "claimed"));
        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Le righe sono aggregate."));

        rest.patch().uri("/api/steps/" + first).body(Map.of("done", true)).retrieve().toEntity(Map.class);

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .as("the wrapup already had this step's work in it when it was written")
                .doesNotContain("finished-since-wrapup")
                .doesNotContain("finished since the wrapup was written")
                .doesNotContain("Somma per settimana.");
    }

    @Test
    @DisplayName("a step moved over MCP reaches an open console over the event stream")
    void theEventStreamCarriesAStepChange() throws Exception {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        aStep(taskId, "Aggregate the rows", null);

        BlockingQueue<String> lines = new LinkedBlockingQueue<>();
        // Not try-with-resources: close() blocks on a stream meant never to finish; shutdownNow() drops it.
        HttpClient client = HttpClient.newHttpClient();
        try {
            client.sendAsync(
                            HttpRequest.newBuilder(URI.create("http://localhost:" + port + "/api/steps/stream"))
                                    .GET().build(),
                            HttpResponse.BodyHandlers.ofLines())
                    .thenAccept(response -> response.body().forEach(lines::add));

            awaitLine(lines, "event:open", 5);

            callTool("rekall_step", Map.of(
                    "anchors", "project:vega task:report-builder", "step", "1", "state", "running"));

            assertThat(awaitLine(lines, "event:steps", 5)).isEqualTo("event:steps");
            assertThat(awaitLine(lines, "data:", 5))
                    .contains(taskId)
                    .contains("\"state\":\"RUNNING\"");
        } finally {
            client.shutdownNow();
        }
    }

    // --- Task-scoped review line (stepless tasks)

    @Test
    @DisplayName("a Claude-authored wrapup claims a task that has no checklist")
    void aClaudeWrapupClaimsASteplessTask() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Lo stato corrente."));

        Map<?, ?> task = getTask(taskId);
        assertThat(task.get("reviewActive")).isEqualTo(true);
        assertThat(task.get("reviewState")).isEqualTo("CLAIMED");
        assertThat(task.get("claimedAt")).isNotNull();
    }

    @Test
    @DisplayName("a hand-written wrapup is the reviewer's correction, so it does not claim the task")
    void aHandWrittenWrapupLeavesASteplessTaskOpen() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        rest.put().uri("/api/tasks/" + taskId + "/wrapup")
                .body(Map.of("bodyMarkdown", "Scritto a mano.")).retrieve().toEntity(Map.class);

        assertThat(getTask(taskId).get("reviewState")).isEqualTo("OPEN");
    }

    @Test
    @DisplayName("the console accepts a stepless task, and cannot accept it twice")
    void theConsoleAcceptsASteplessTask() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        ResponseEntity<Map> accepted = review(taskId, Map.of("reviewState", "DONE"));
        assertThat(accepted.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(accepted.getBody().get("reviewState")).isEqualTo("DONE");
        assertThat(accepted.getBody().get("acceptedAt")).isNotNull();

        assertThat(review(taskId, Map.of("reviewState", "DONE")).getStatusCode())
                .as("a second accept has nothing to do and says so")
                .isEqualTo(HttpStatus.BAD_REQUEST);
    }

    @Test
    @DisplayName("sending a stepless task back reopens it and carries a note to the next session")
    void theConsoleSendsASteplessTaskBackWithANote() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Lo stato."));

        ResponseEntity<Map> sentBack = review(taskId, Map.of(
                "reviewState", "OPEN", "note", "la colonna export e' ancora sbagliata"));

        assertThat(sentBack.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(sentBack.getBody().get("reviewState")).isEqualTo("OPEN");
        assertThat(sentBack.getBody().get("reviewNote")).isEqualTo("la colonna export e' ancora sbagliata");
        assertThat(sentBack.getBody().get("claimedAt")).isNull();
    }

    @Test
    @DisplayName("adding a checklist retires the task-level review line")
    void aChecklistRetiresTheTaskLevelReviewLine() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");
        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "Lo stato."));

        aStep(taskId, "Aggregate the rows", null);

        assertThat(getTask(taskId).get("reviewActive")).isEqualTo(false);
        assertThat(review(taskId, Map.of("reviewState", "DONE")).getStatusCode())
                .as("the steps carry the review now, not the task")
                .isEqualTo(HttpStatus.BAD_REQUEST);
    }

    @Test
    @DisplayName("the review endpoint refuses the two states nothing outside the system may set")
    void theReviewEndpointRefusesTheDerivedStates() {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        assertThat(review(taskId, Map.of("reviewState", "RUNNING")).getStatusCode())
                .isEqualTo(HttpStatus.BAD_REQUEST);
        assertThat(review(taskId, Map.of("reviewState", "CLAIMED")).getStatusCode())
                .isEqualTo(HttpStatus.BAD_REQUEST);
    }

    @Test
    @DisplayName("a stepless task accepted in the console reaches an open console over the event stream")
    void theEventStreamCarriesAReviewChange() throws Exception {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        BlockingQueue<String> lines = new LinkedBlockingQueue<>();
        HttpClient client = HttpClient.newHttpClient();
        try {
            client.sendAsync(
                            HttpRequest.newBuilder(URI.create("http://localhost:" + port + "/api/steps/stream"))
                                    .GET().build(),
                            HttpResponse.BodyHandlers.ofLines())
                    .thenAccept(response -> response.body().forEach(lines::add));

            awaitLine(lines, "event:open", 5);
            review(taskId, Map.of("reviewState", "DONE"));

            assertThat(awaitLine(lines, "event:task-review", 5)).isEqualTo("event:task-review");
            assertThat(awaitLine(lines, "data:", 5))
                    .contains(taskId)
                    .contains("\"reviewState\":\"DONE\"");
        } finally {
            client.shutdownNow();
        }
    }

    @Test
    @DisplayName("a wrapup written and then deleted reaches an open console over the event stream")
    void theEventStreamCarriesAWrapupChange() throws Exception {
        String acme = aCompany("Acme");
        String projectId = aProject(acme, "vega", "ACTIVE");
        String taskId = aTask(projectId, "report-builder");

        BlockingQueue<String> lines = new LinkedBlockingQueue<>();
        HttpClient client = HttpClient.newHttpClient();
        try {
            client.sendAsync(
                            HttpRequest.newBuilder(URI.create("http://localhost:" + port + "/api/steps/stream"))
                                    .GET().build(),
                            HttpResponse.BodyHandlers.ofLines())
                    .thenAccept(response -> response.body().forEach(lines::add));

            awaitLine(lines, "event:open", 5);

            callTool("rekall_wrapup", Map.of(
                    "anchors", "project:vega task:report-builder", "body", "Lo stato corrente."));

            assertThat(awaitLine(lines, "event:wrapup", 5)).isEqualTo("event:wrapup");
            assertThat(awaitLine(lines, "data:", 5))
                    .contains(taskId)
                    .contains("Lo stato corrente.")
                    .contains("\"writtenBy\":\"CLAUDE\"")
                    .contains("\"deleted\":false");

            rest.delete().uri("/api/tasks/" + taskId + "/wrapup").retrieve().toEntity(Void.class);

            assertThat(awaitLine(lines, "event:wrapup", 5)).isEqualTo("event:wrapup");
            assertThat(awaitLine(lines, "data:", 5))
                    .contains(taskId)
                    .contains("\"deleted\":true");
        } finally {
            client.shutdownNow();
        }
    }

    @Test
    @DisplayName("the export is a zip of company/project/task/note.md, shared notes under each task")
    void exportsAFolderTree() throws Exception {
        String acme = aCompany("Acme");
        String vega = id(post("/api/projects", Map.of(
                "label", "vega", "title", "Vega Platform", "status", "ACTIVE", "companyId", acme)));
        String validator = aTask(vega, "report-builder");
        String retry = aTask(vega, "retry-policy");
        // A task with no notes still has to appear.
        aTask(vega, "empty-one");

        post("/api/documents", Map.of("title", "CONTEXT.md", "kind", "context",
                "taskIds", List.of(validator), "bodyMarkdown", "# Contesto\n\nprimo"));
        post("/api/documents", Map.of("title", "kmaster14.md", "kind", "notes",
                "taskIds", List.of(validator, retry), "bodyMarkdown", "Accesso via bastion."));
        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "## Stato\n\nGira."));
        String aggregate = aStep(validator, "Aggregate the rows", null);
        aStep(validator, "Write the tests", "Un caso per settimana vuota.");
        rest.patch().uri("/api/steps/" + aggregate).body(Map.of("done", true))
                .retrieve().toEntity(Map.class);

        Map<String, String> entries = unzip(
                rest.get().uri("/api/export").retrieve().toEntity(byte[].class).getBody());

        assertThat(entries.keySet())
                .as("folders are named after the label, which is what the anchor says")
                .contains(
                        "Acme/vega/report-builder/CONTEXT.md",
                        "Acme/vega/report-builder/kmaster14.md",
                        "Acme/vega/retry-policy/kmaster14.md",
                        "Acme/vega/empty-one/",
                        "MANIFEST.md")
                .as("the wrapup travels as the file you open first, and the checklist beside it")
                .contains("Acme/vega/report-builder/WRAPUP.md", "Acme/vega/report-builder/STEPS.md");

        assertThat(entries.get("Acme/vega/report-builder/STEPS.md"))
                .as("as a checklist, whole: this is a disk, not a context window")
                .contains("- [x] Aggregate the rows")
                .contains("- [ ] Write the tests")
                .contains("Un caso per settimana vuota.");

        assertThat(entries.get("Acme/vega/report-builder/CONTEXT.md")).isEqualTo("# Contesto\n\nprimo");
        assertThat(entries.get("Acme/vega/retry-policy/kmaster14.md"))
                .as("the shared note is complete in every folder, not a pointer")
                .isEqualTo("Accesso via bastion.");

        assertThat(entries.get("MANIFEST.md"))
                .contains("company:Acme")
                .as("the manifest carries the title and the anchor that reaches it")
                .contains("Vega Platform")
                .contains("project:vega task:report-builder")
                .as("and says which files are copies of one note")
                .contains("Notes that appear more than once")
                .contains("Acme/vega/retry-policy/kmaster14.md")
                .as("and reports the task that has none")
                .contains("no notes")
                .as("and names the wrapup, and who wrote it")
                .contains("`WRAPUP.md`: the state of the implementation, written by Claude")
                .as("and how far the checklist has got")
                .contains("`STEPS.md`: 1 of 2 steps done");
    }

    @Test
    @DisplayName("a label full of separators cannot escape its folder")
    void exportRefusesToBuildPathsFromNames() throws Exception {
        String acme = aCompany("Acme");
        String project = aProject(acme, "../../etc", "ACTIVE");
        String task = aTask(project, "a/b/c");
        post("/api/documents", Map.of("title", "../secret", "kind", "notes",
                "taskIds", List.of(task), "bodyMarkdown", "x"));

        Map<String, String> entries = unzip(
                rest.get().uri("/api/export").retrieve().toEntity(byte[].class).getBody());

        assertThat(entries.keySet())
                .allSatisfy(path -> assertThat(path).doesNotContain("..").doesNotStartWith("/"));
        assertThat(entries.keySet()).anySatisfy(path -> assertThat(path).endsWith("secret.md"));
    }

    @Test
    @DisplayName("a project label two companies share is disambiguated by the company anchor")
    void projectLabelSharedAcrossCompanies() {
        String acme = aCompany("Acme");
        String globex = aCompany("Globex");
        aProject(acme, "website", "ACTIVE");
        aProject(globex, "website", "PAUSED");

        assertThat(callTool("rekall_context", Map.of("anchors", "project:website")))
                .contains("matches 2 records")
                .contains("company:Acme")
                .contains("company:Globex");
    }

    @Test
    @DisplayName("a company anchor lists its projects, and a task reaches back up to it")
    void companyAnchorAndUpwardPath() {
        String acme = aCompany("Acme");
        String vega = aProject(acme, "vega", "ACTIVE");
        aTask(vega, "report-builder");

        assertThat(callTool("rekall_context", Map.of("anchors", "company:Acme")))
                .contains("Company: Acme")
                .contains("`project:vega`");

        assertThat(callTool("rekall_context", Map.of("anchors", "task:report-builder")))
                .as("a task carries its project, and the project carries its company")
                .contains("Task: report-builder")
                .contains("Project: vega")
                .contains("Company: Acme");
    }

    @Test
    @DisplayName("deleting a company takes its projects and tasks, and sweeps the orphaned notes")
    void deletingACompanyCascades() {
        String acme = aCompany("Acme");
        String vega = aProject(acme, "vega", "ACTIVE");
        String task = aTask(vega, "t");
        post("/api/documents", Map.of("title", "n.md", "kind", "notes",
                "taskIds", List.of(task), "bodyMarkdown", "x"));

        rest.delete().uri("/api/companies/" + acme).retrieve().toEntity(Void.class);

        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM project", Integer.class)).isZero();
        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM task", Integer.class)).isZero();
        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM document", Integer.class))
                .as("the notes had nothing left to hang from")
                .isZero();
    }

    // Asserts the migration's normalising expression itself, on the database that will run it.
    @ParameterizedTest
    @CsvSource({"Vega, vega", "'Progetto Vega', progetto-vega", "'../../etc', etc", "a/b/c, a-b-c"})
    @DisplayName("the migration's normalising expression turns a legacy name into a label")
    void legacyNamesAreNormalisedBySql(String legacy, String expected) {
        assertThat(jdbc.queryForObject(
                "SELECT REGEXP_REPLACE(REGEXP_REPLACE(LOWER(?), '[^a-z0-9._-]+', '-'),"
                        + " '^[._-]+|[._-]+$', '')",
                String.class, legacy))
                .isEqualTo(expected);
    }

    @Test
    @DisplayName("a project cannot exist without a company")
    void projectNeedsACompany() {
        assertThat(post("/api/projects", Map.of("label", "orphan", "title", "Orphan", "status", "ACTIVE"))
                        .getStatusCode())
                .isEqualTo(HttpStatus.NOT_FOUND);
    }

    // ------------------------------------------------------------------ helpers

    private String awaitLine(BlockingQueue<String> lines, String needle, int seconds) throws InterruptedException {
        long deadline = System.nanoTime() + seconds * 1_000_000_000L;
        while (System.nanoTime() < deadline) {
            String line = lines.poll(200, TimeUnit.MILLISECONDS);
            if (line != null && line.contains(needle)) {
                return line;
            }
        }
        throw new AssertionError("No event-stream line containing '" + needle + "' within " + seconds + "s");
    }

    /** Entry name to contents; directory entries kept as their own empty entries. */
    private Map<String, String> unzip(byte[] archive) throws Exception {
        Map<String, String> entries = new LinkedHashMap<>();
        try (ZipInputStream zip = new ZipInputStream(new ByteArrayInputStream(archive), StandardCharsets.UTF_8)) {
            for (ZipEntry entry = zip.getNextEntry(); entry != null; entry = zip.getNextEntry()) {
                entries.put(entry.getName(), new String(zip.readAllBytes(), StandardCharsets.UTF_8));
            }
        }
        return entries;
    }


    private String aCompany(String name) {
        return id(post("/api/companies", Map.of("name", name)));
    }

    private String aProject(String companyId, String label, String status) {
        return id(post("/api/projects", Map.of(
                "label", label, "title", label, "status", status, "companyId", companyId)));
    }

    private String aTask(String projectId, String label) {
        return id(post("/api/tasks", Map.of(
                "label", label, "title", label, "status", "TODO", "projectId", projectId)));
    }

    /** Creates a step and promotes it out of draft. */
    private String aStep(String taskId, String title, String bodyMarkdown) {
        Map<String, Object> body = new LinkedHashMap<>();
        body.put("title", title);
        body.put("bodyMarkdown", bodyMarkdown);
        String stepId = id(post("/api/tasks/" + taskId + "/steps", body));
        rest.patch().uri("/api/steps/" + stepId).body(Map.of("draft", false))
                .retrieve().toEntity(Map.class);
        return stepId;
    }

    private String aDraftStep(String taskId, String title, String bodyMarkdown) {
        Map<String, Object> body = new LinkedHashMap<>();
        body.put("title", title);
        body.put("bodyMarkdown", bodyMarkdown);
        return id(post("/api/tasks/" + taskId + "/steps", body));
    }

    @SuppressWarnings("rawtypes")
    private ResponseEntity<Map> post(String path, Object body) {
        return rest.post().uri(path).body(body).retrieve().toEntity(Map.class);
    }

    @SuppressWarnings("rawtypes")
    private Map<?, ?> getTask(String taskId) {
        return rest.get().uri("/api/tasks/" + taskId).retrieve().toEntity(Map.class).getBody();
    }

    @SuppressWarnings("rawtypes")
    private ResponseEntity<Map> review(String taskId, Map<String, Object> body) {
        return rest.patch().uri("/api/tasks/" + taskId + "/review").body(body)
                .retrieve().toEntity(Map.class);
    }

    private String id(ResponseEntity<Map> response) {
        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
        return String.valueOf(response.getBody().get("id"));
    }

    @SuppressWarnings("rawtypes")
    private List<String> labelsOfSteps(String taskId) {
        List<?> steps = rest.get().uri("/api/tasks/" + taskId + "/steps")
                .retrieve().toEntity(List.class).getBody();
        return steps.stream().map(step -> String.valueOf(((Map<?, ?>) step).get("title"))).toList();
    }

    @SuppressWarnings("rawtypes")
    private List<?> documentsOn(String taskId) {
        return rest.get().uri("/api/documents?taskId=" + taskId).retrieve().toEntity(List.class).getBody();
    }

    private String callTool(String name, Map<String, Object> arguments) {
        Map<?, ?> result = (Map<?, ?>)
                rpc("tools/call", Map.of("name", name, "arguments", arguments)).get("result");
        List<?> content = (List<?>) result.get("content");
        return String.valueOf(((Map<?, ?>) content.getFirst()).get("text"));
    }

    /** A stateless-era request: no handshake, every field mirrored into a header. */
    @SuppressWarnings("rawtypes")
    private ResponseEntity<Map> modernRpc(String method, String name, Object params) {
        RestClient.RequestBodySpec spec = rest.post()
                .uri("/mcp")
                .header("MCP-Protocol-Version", "2026-07-28")
                .header("Mcp-Method", method);
        if (name != null) {
            spec = spec.header("Mcp-Name", name);
        }
        return spec.body(Map.of("jsonrpc", "2.0", "id", 1, "method", method, "params", params))
                .retrieve()
                .toEntity(Map.class);
    }

    @SuppressWarnings("rawtypes")
    private Map<?, ?> rpc(String method, Object params) {
        return rest.post()
                .uri("/mcp")
                .body(Map.of("jsonrpc", "2.0", "id", 1, "method", method, "params", params))
                .retrieve()
                .toEntity(Map.class)
                .getBody();
    }
}
