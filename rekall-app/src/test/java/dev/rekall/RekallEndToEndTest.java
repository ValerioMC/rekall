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
import org.springframework.web.client.RestClient;

import java.util.List;
import java.util.Map;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The whole application, from entering data through the UI's API to answering with it over MCP.
 *
 * <p>The scenario is the real one: a project, a task on it, and the environment the task runs
 * against. If this passes, {@code /rk project:stvv task:code-validator-main-workflow} loads a
 * usable working context in one call.
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
        jdbc.execute("DELETE FROM document");
        jdbc.execute("DELETE FROM task");
        jdbc.execute("DELETE FROM project");
        jdbc.execute("DELETE FROM environment");
    }

    @Test
    @DisplayName("one anchored call brings back the task, its project, its environment and every note")
    void loadsTheWholeWorkingContext() {
        String environmentId = id(post("/api/environments", Map.of(
                "label", "kmaster14 / stvv-dev",
                "namespace", "stvv-dev",
                "kubeconfigPath", "/Users/valeriomc/Projects/ESA/_kmasters/config.kmaster14")));
        String projectId = id(post("/api/projects", Map.of(
                "name", "STVV", "status", "ACTIVE", "description", "Progetto STVV")));
        post("/api/projects", Map.of("name", "AINABLER", "status", "PAUSED"));
        String taskId = id(post("/api/tasks", Map.of(
                "name", "code-validator-main-workflow",
                "status", "IN_PROGRESS",
                "projectId", projectId,
                "environmentId", environmentId)));

        post("/api/documents", Map.of(
                "title", "CONTEXT.md", "kind", "context", "taskId", taskId,
                "bodyMarkdown", "# Contesto\n\nIl workflow parte da POST /api/v1/pipelines."));
        // The note that lived in cluster.md and never made it into a conversation before.
        post("/api/documents", Map.of(
                "title", "cluster.md", "kind", "notes", "environmentId", environmentId,
                "bodyMarkdown", "Cluster kmaster14, accesso via bastion."));

        String context = callTool("rekall_context", Map.of(
                "anchors", "project:STVV task:code-validator-main-workflow"));

        assertThat(context)
                .contains("Project: STVV")
                .contains("Task: code-validator-main-workflow")
                .as("the environment must arrive resolved, not as an id")
                .contains("kmaster14 / stvv-dev")
                .contains("stvv-dev")
                .as("the task's own notes arrive in full")
                .contains("POST /api/v1/pipelines")
                .as("and so do the notes of what the task references, which is the whole point")
                .contains("Cluster kmaster14, accesso via bastion.")
                .doesNotContain("AINABLER");

        // Reached twice, written once: the project is both an anchor and the task's reference.
        assertThat(context.split("Project: STVV", -1)).as("no record is rendered twice").hasSize(2);
    }

    @Test
    @DisplayName("a single project anchor lists its tasks as anchors, without their bodies")
    void projectAnchorListsTasks() {
        String projectId = id(post("/api/projects", Map.of("name", "STVV", "status", "ACTIVE")));
        String taskId = id(post("/api/tasks", Map.of(
                "name", "code-validator-main-workflow", "status", "TODO", "projectId", projectId)));
        post("/api/documents", Map.of(
                "title", "CONTEXT.md", "kind", "context", "taskId", taskId, "bodyMarkdown", "segreto"));

        String context = callTool("rekall_context", Map.of("anchors", "project:STVV"));

        assertThat(context)
                .contains("`task:code-validator-main-workflow`")
                .as("an inverse listing carries anchors only, never the referenced bodies")
                .doesNotContain("segreto");
    }

    @Test
    @DisplayName("a bare term resolves when it is unambiguous and reports the candidates when it is not")
    void positionalAnchors() {
        String projectId = id(post("/api/projects", Map.of("name", "STVV", "status", "ACTIVE")));
        post("/api/tasks", Map.of("name", "STVV", "status", "TODO", "projectId", projectId));

        assertThat(callTool("rekall_context", Map.of("anchors", "code-validator")))
                .contains("Nothing matches 'code-validator'");
        assertThat(callTool("rekall_context", Map.of("anchors", "STVV")))
                .as("a project and a task both called STVV must stop the tool, not be guessed at")
                .contains("matches 2 records")
                .contains("Qualify it as `entity:value`");
    }

    @Test
    @DisplayName("a task name that two projects share is disambiguated by the project anchor")
    void taskNameSharedAcrossProjects() {
        String stvv = id(post("/api/projects", Map.of("name", "STVV", "status", "ACTIVE")));
        String ainabler = id(post("/api/projects", Map.of("name", "AINABLER", "status", "ACTIVE")));
        post("/api/tasks", Map.of("name", "setup", "status", "TODO", "projectId", stvv));
        post("/api/tasks", Map.of("name", "setup", "status", "DONE", "projectId", ainabler));

        assertThat(callTool("rekall_context", Map.of("anchors", "task:setup")))
                .contains("matches 2 records");
        assertThat(callTool("rekall_context", Map.of("anchors", "project:AINABLER task:setup")))
                .contains("Task: setup")
                .contains("DONE");
    }

    @Test
    @DisplayName("the MCP endpoint exposes exactly one tool")
    void toolsList() {
        List<?> tools = (List<?>) ((Map<?, ?>) rpc("tools/list", Map.of()).get("result")).get("tools");

        assertThat(tools).hasSize(1);
        assertThat(((Map<?, ?>) tools.getFirst()).get("name")).isEqualTo("rekall_context");
    }

    @Test
    @DisplayName("deleting an environment tasks still run on is refused as a conflict")
    void deletingUsedEnvironmentIsAConflict() {
        String environmentId = id(post("/api/environments", Map.of("label", "kmaster14 / stvv-dev")));
        String projectId = id(post("/api/projects", Map.of("name", "STVV", "status", "ACTIVE")));
        post("/api/tasks", Map.of(
                "name", "t", "status", "TODO", "projectId", projectId, "environmentId", environmentId));

        ResponseEntity<Map> response =
                rest.delete().uri("/api/environments/" + environmentId).retrieve().toEntity(Map.class);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CONFLICT);
        assertThat(String.valueOf(response.getBody().get("detail"))).contains("still run on");
    }

    @Test
    @DisplayName("a note belongs to exactly one owner, and the API refuses anything else")
    void documentNeedsExactlyOneOwner() {
        String projectId = id(post("/api/projects", Map.of("name", "STVV", "status", "ACTIVE")));

        assertThat(post("/api/documents", Map.of("title", "x", "kind", "notes")).getStatusCode())
                .isEqualTo(HttpStatus.CONFLICT);
        assertThat(post("/api/documents", Map.of("title", "x", "kind", "notes", "projectId", projectId))
                        .getStatusCode())
                .isEqualTo(HttpStatus.CREATED);
    }

    // ------------------------------------------------------------------ helpers

    @SuppressWarnings("rawtypes")
    private ResponseEntity<Map> post(String path, Object body) {
        return rest.post().uri(path).body(body).retrieve().toEntity(Map.class);
    }

    private String id(ResponseEntity<Map> response) {
        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
        return String.valueOf(response.getBody().get("id"));
    }

    private String callTool(String name, Map<String, Object> arguments) {
        Map<?, ?> result = (Map<?, ?>)
                rpc("tools/call", Map.of("name", name, "arguments", arguments)).get("result");
        List<?> content = (List<?>) result.get("content");
        return String.valueOf(((Map<?, ?>) content.getFirst()).get("text"));
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
