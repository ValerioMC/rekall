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
import org.springframework.http.ResponseEntity;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.web.client.RestClient;

import java.io.ByteArrayInputStream;
import java.nio.charset.StandardCharsets;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The whole application, from entering data through the UI's API to answering with it over MCP.
 *
 * <p>The scenario is the real one: a project, tasks on it, and the notes those tasks share. If
 * this passes, {@code /rk project:vega task:report-builder} loads a usable working context in
 * one call.
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
        // Status handling is disabled so the tests can assert on codes rather than on thrown
        // exceptions. TestRestTemplate no longer ships with Spring Boot 4.
        rest = RestClient.builder()
                .baseUrl("http://localhost:" + port)
                .defaultStatusHandler(status -> true, (request, response) -> { })
                .build();
    }

    @BeforeEach
    void resetDatabase() {
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
        // The note that lived in cluster.md and never made it into a conversation before.
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

        // Reached twice, written once: the project is both an anchor and the task's reference.
        assertThat(context.split("Project: Vega Platform", -1)).as("no record is rendered twice").hasSize(2);
    }

    /**
     * A description is a markdown document, saying what the work is, what it has to satisfy and
     * what is out of scope, so it is handed over in a tag of its own. Rendered into the field list
     * above it, every line after the first would fall outside the bullet and its headings would
     * outrank the record's.
     */
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

    /**
     * The reason the two fields are separate columns. An anchor is written down in a slash
     * command, in a note, in someone's head; a title is rewritten the moment a better name comes
     * along. Renaming must not break what was written down.
     */
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

    /**
     * What a person types into the label field is a name, and what the anchor needs is an
     * identifier. Narrowed on the way in rather than rejected, because the two differ by
     * punctuation and nothing else.
     */
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

    /** An anchor that names two records loads neither, so the second one is refused. */
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

    /**
     * The folder a session is opened in is the one thing a task cannot work out for itself, and
     * the button that opens one lives on the task. So it is stored on the project and travels
     * down with every task the project holds.
     */
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

        // Cleared in the interface is an empty field, and an empty path is not a path.
        assertThat(updateProjectFolder(projectId, acme, "   ").get("repoFolder")).isNull();
    }

    @SuppressWarnings("unchecked")
    private Map<String, Object> updateProjectFolder(String projectId, String companyId, String folder) {
        return rest.put().uri("/api/projects/" + projectId)
                .body(Map.of("label", "vega", "title", "Vega", "status", "ACTIVE",
                        "companyId", companyId, "repoFolder", folder))
                .retrieve().toEntity(Map.class).getBody();
    }

    /**
     * The reason the relation was changed. One note, three tasks, one copy: editing it once
     * changes what every one of those tasks loads.
     */
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

        // Editing through one task is visible from the other: there is one row, not two.
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

    /**
     * A note on other tasks survives; a note left on nothing is swept up. The join table cannot
     * express the second rule, because a row that no longer exists cannot be checked.
     */
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

    /** Deleting a project takes its tasks, and the interface has to say so before it happens. */
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

    /**
     * Two tools and no more: one way in, one thing that may be written. Asserted as an exact
     * list rather than a count, because the value of "there is no query tool and no get tool" is
     * only kept if adding one breaks a test.
     */
    @Test
    @DisplayName("the MCP endpoint exposes one way to read and one thing to write")
    void toolsList() {
        List<?> tools = (List<?>) ((Map<?, ?>) rpc("tools/list", Map.of()).get("result")).get("tools");

        assertThat(tools.stream().map(tool -> String.valueOf(((Map<?, ?>) tool).get("name"))))
                .containsExactlyInAnyOrder("rekall_context", "rekall_wrapup");
    }

    /*
     * The wrapup. A task records what its implementation currently is, Claude writes it at the
     * end of a session and reads it at the start of the next one, and the console corrects it.
     */

    /**
     * The loop the whole feature exists for: written over MCP, loaded back by the anchor that
     * wrote it, without anyone naming a file.
     */
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

    /**
     * One per task, and it is the database that says so. A second write is a replacement, which
     * is what makes a wrapup a description of the state rather than a log of the session.
     */
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

    /**
     * Overwriting a correction someone made by hand is the one outcome worth saying out loud,
     * because the words that disappear are theirs and nothing keeps a copy.
     */
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

    /** A write has to land on one task and be sure of it, so anything vaguer is refused. */
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
                .contains("A wrapup belongs to exactly one task");

        assertThat(callTool("rekall_wrapup", Map.of("anchors", "task:setup", "body", "x")))
                .contains("matches 2 records")
                .contains("Qualify it with `project:");

        assertThat(callTool("rekall_wrapup", Map.of("anchors", "task:nowhere", "body", "x")))
                .contains("No task matches 'nowhere'");

        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM wrapup", Integer.class))
                .as("nothing was written on any of those")
                .isZero();
    }

    /**
     * The cap is where a wrapup that has turned into a log gets caught, and the refusal says
     * which of the two it became.
     */
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

        // Caught by the argument reader before the service sees it, which is the earliest place
        // it can be, and answered as tool content Claude can retry from.
        assertThat(callTool("rekall_wrapup", Map.of(
                        "anchors", "project:vega task:report-builder", "body", "   ")))
                .contains("'body' is required");
        assertThat(jdbc.queryForObject("SELECT COUNT(*) FROM wrapup", Integer.class)).isZero();
    }

    /** A wrapup describes one task and has no meaning without it. */
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

    /** The task list is what the navigator reads, and it has to say which tasks have said. */
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

    /*
     * Time entries. A task is worked in sittings, each one its own session, and at most one
     * session across the whole application is open at a time.
     */

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

    /** Different tasks track in parallel: starting one never touches what is running elsewhere. */
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

    /** A doubled click, or a race on the button, must not open a second session. */
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

    /** A session describes a task and has no meaning without it. */
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

    /*
     * The 2026-07-28 era, over real HTTP. McpProtocolTest drives the controller directly, so
     * this is the only place that proves Spring binds the mirrored headers at all: a header name
     * the framework fails to map arrives as null, which the controller cannot tell apart from a
     * client that never sent it, and every one of those tests would still pass.
     */

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

    /**
     * Flushed before mapping, because the timestamps are written by Hibernate at flush time.
     * Returning the entity earlier left {@code updatedAt} null, and the frontend rejected a
     * create that had in fact succeeded.
     */
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

    /** A task without a title has no name to show, so it is refused before it reaches the table. */
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

    /**
     * The forwarding list drifted once already: it still named the screens of the deleted
     * schema designer, so every current route answered 404 on a refresh while navigating inside
     * the application worked perfectly.
     */
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

    /** Environments were removed with the model, so their endpoints must be gone too. */
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

    /**
     * The export is a backup and an escape hatch: a tree of folders that outlives the
     * application. What it cannot represent is a note on several tasks, so it writes the note
     * under each of them and says so in the manifest.
     */
    @Test
    @DisplayName("the export is a zip of company/project/task/note.md, shared notes under each task")
    void exportsAFolderTree() throws Exception {
        String acme = aCompany("Acme");
        String vega = id(post("/api/projects", Map.of(
                "label", "vega", "title", "Vega Platform", "status", "ACTIVE", "companyId", acme)));
        String validator = aTask(vega, "report-builder");
        String retry = aTask(vega, "retry-policy");
        // A task with no notes still has to appear, or the tree misreports the work.
        aTask(vega, "empty-one");

        post("/api/documents", Map.of("title", "CONTEXT.md", "kind", "context",
                "taskIds", List.of(validator), "bodyMarkdown", "# Contesto\n\nprimo"));
        post("/api/documents", Map.of("title", "kmaster14.md", "kind", "notes",
                "taskIds", List.of(validator, retry), "bodyMarkdown", "Accesso via bastion."));
        callTool("rekall_wrapup", Map.of(
                "anchors", "project:vega task:report-builder", "body", "## Stato\n\nGira."));

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
                .as("the wrapup travels as the file you open first")
                .contains("Acme/vega/report-builder/WRAPUP.md");

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
                .contains("`WRAPUP.md`: the state of the implementation, written by Claude");
    }

    /** A record named after a path must become a folder name, never a path. */
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

    /**
     * A project label only has to be unique inside its company, so the bare anchor can now match
     * twice. Reported rather than guessed at, the same way task labels already were.
     */
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

    /** The blast radius the interface has to state before anyone clicks it. */
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

    /**
     * The migration that split name into label and title also has to repair the values, and it
     * runs against an empty schema here, so nothing else would notice if the expression silently
     * matched nothing. This asserts the expression itself, on the database that will run it.
     */
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

    /** Entry name to contents, with directory entries kept as their own empty entries. */
    private Map<String, String> unzip(byte[] archive) throws Exception {
        Map<String, String> entries = new LinkedHashMap<>();
        try (ZipInputStream zip = new ZipInputStream(new ByteArrayInputStream(archive), StandardCharsets.UTF_8)) {
            for (ZipEntry entry = zip.getNextEntry(); entry != null; entry = zip.getNextEntry()) {
                entries.put(entry.getName(), new String(zip.readAllBytes(), StandardCharsets.UTF_8));
            }
        }
        return entries;
    }


    /** Every project needs a company, so the scenarios open with one. */
    private String aCompany(String name) {
        return id(post("/api/companies", Map.of("name", name)));
    }

    /** Where the title carries no weight in the scenario, it is the label written out again. */
    private String aProject(String companyId, String label, String status) {
        return id(post("/api/projects", Map.of(
                "label", label, "title", label, "status", status, "companyId", companyId)));
    }

    private String aTask(String projectId, String label) {
        return id(post("/api/tasks", Map.of(
                "label", label, "title", label, "status", "TODO", "projectId", projectId)));
    }

    @SuppressWarnings("rawtypes")
    private ResponseEntity<Map> post(String path, Object body) {
        return rest.post().uri(path).body(body).retrieve().toEntity(Map.class);
    }

    private String id(ResponseEntity<Map> response) {
        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
        return String.valueOf(response.getBody().get("id"));
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

    /** A request in the stateless era: no handshake, and every field mirrored into a header. */
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
