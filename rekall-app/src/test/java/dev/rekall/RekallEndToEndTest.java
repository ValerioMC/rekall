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
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.testcontainers.containers.PostgreSQLContainer;

import java.util.List;
import java.util.Map;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The whole application, from designing a schema in the UI's API to answering a question
 * through MCP.
 *
 * <p>The scenario is deliberately the real one: a project, its tasks, and an environment the
 * tasks point at. If this passes, "Continuiamo a lavorare sul progetto STVV nel task
 * code-validator-main-workflow" works.
 */
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class RekallEndToEndTest {

    private static final PostgreSQLContainer<?> POSTGRES = new PostgreSQLContainer<>("postgres:16-alpine")
            .withDatabaseName("rekall")
            .withUsername("rekall")
            .withPassword("rekall");

    static {
        POSTGRES.start();
    }

    @DynamicPropertySource
    static void datasource(DynamicPropertyRegistry registry) {
        registry.add("spring.datasource.url", POSTGRES::getJdbcUrl);
        registry.add("spring.datasource.username", POSTGRES::getUsername);
        registry.add("spring.datasource.password", POSTGRES::getPassword);
    }

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
        jdbc.execute(
                """
                DO $$
                DECLARE r record;
                BEGIN
                    FOR r IN SELECT tablename FROM pg_tables WHERE schemaname = 'rekall_data' LOOP
                        EXECUTE format('DROP TABLE IF EXISTS rekall_data.%I CASCADE', r.tablename);
                    END LOOP;
                END
                $$;
                """);
        jdbc.update("DELETE FROM rekall_meta.document");
        jdbc.update("DELETE FROM rekall_meta.ddl_log");
        jdbc.update("DELETE FROM rekall_meta.meta_relation");
        jdbc.update("UPDATE rekall_meta.meta_table SET display_field_id = NULL");
        jdbc.update("DELETE FROM rekall_meta.meta_field");
        jdbc.update("DELETE FROM rekall_meta.meta_table");
    }

    @Test
    @DisplayName("design a schema, apply it, fill it, then answer questions about it through MCP")
    void fullJourney() {
        // 1. Design the entities the way the UI would.
        String environmentId = createTable("environment", "Environment", "Environments",
                "Configurazioni di cluster, namespace e database", List.of("ambienti"));
        String environmentLabelField = addField(environmentId, "label", "Label", "Nome leggibile dell'ambiente", "TEXT");
        addField(environmentId, "namespace", "Namespace", "Namespace Kubernetes", "TEXT");
        setDisplayField(environmentId, "environment", "Environment", "Environments",
                "Configurazioni di cluster, namespace e database", List.of("ambienti"), environmentLabelField);

        String projectId = createTable("project", "Project", "Projects",
                "Tutti i progetti su cui si sta lavorando", List.of("progetti", "commesse"));
        String projectNameField = addField(projectId, "name", "Name", "Nome breve del progetto", "TEXT");
        addEnumField(projectId, "status", "Status", "Stato corrente", List.of("active", "paused", "done"));
        setDisplayField(projectId, "project", "Project", "Projects",
                "Tutti i progetti su cui si sta lavorando", List.of("progetti", "commesse"), projectNameField);

        String taskId = createTable("task", "Task", "Tasks",
                "Attivita e issue legate a un progetto", List.of("issues", "attivita"));
        String taskNameField = addField(taskId, "name", "Name", "Identificativo del task", "TEXT");
        String taskProjectField = addField(taskId, "project_id", "Project", "Il progetto a cui appartiene", "REFERENCE");
        String taskEnvironmentField =
                addField(taskId, "environment_id", "Environment", "L'ambiente su cui gira", "REFERENCE");
        setDisplayField(taskId, "task", "Task", "Tasks",
                "Attivita e issue legate a un progetto", List.of("issues", "attivita"), taskNameField);

        createRelation(taskId, projectId, taskProjectField, "Il progetto a cui il task appartiene");
        createRelation(taskId, environmentId, taskEnvironmentField, "L'ambiente su cui il task gira");

        // 2. The plan is reviewable and nothing has happened to the database yet.
        Map<?, ?> plan = rest.get().uri("/api/meta/plan").retrieve().body(Map.class);
        assertThat(plan.get("applicable")).isEqualTo(true);
        assertThat((List<?>) plan.get("statements")).isNotEmpty();
        assertThat(tableCount()).as("planning must not create anything").isZero();

        // 3. Apply.
        ResponseEntity<Map> applied =
                post("/api/meta/apply", Map.of("backfillDefaults", Map.of(), "confirmedDrops", List.of()));
        assertThat(applied.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(tableCount()).isEqualTo(3);

        // 4. Fill it, exactly as the data browser would.
        String environmentRecord = createRecord("environment",
                Map.of("label", "kmaster14 / stvv-dev", "namespace", "stvv-dev"));
        String projectRecord = createRecord("project", Map.of("name", "STVV", "status", "active"));
        createRecord("project", Map.of("name", "AINABLER", "status", "paused"));
        String taskRecord = createRecord("task", Map.of(
                "name", "code-validator-main-workflow",
                "project_id", projectRecord,
                "environment_id", environmentRecord));

        post(
                "/api/documents",
                Map.of(
                        "entityName", "task",
                        "recordId", taskRecord,
                        "title", "CONTEXT.md",
                        "kind", "context",
                        "bodyMarkdown", "# Contesto\n\nIl workflow parte da POST /api/v1/pipelines su kmaster14."));

        // 5. Ask the questions the application exists for.
        String schema = callTool("rekall_schema", Map.of());
        assertThat(schema)
                .contains("Tutti i progetti su cui si sta lavorando")
                .contains("progetti, commesse")
                .contains("belongs to one `project`")
                .as("the inverse direction has to be visible or a project cannot reach its tasks")
                .contains("has many `task`");

        // The one call a session starts with: /rk project:stvv task:code-validator-main-workflow.
        String context = callTool(
                "rekall_context", Map.of("anchors", "project:STVV task:code-validator-main-workflow"));
        assertThat(context)
                .contains("code-validator-main-workflow")
                .as("the environment config must arrive with the task, not as a bare id")
                .contains("kmaster14 / stvv-dev")
                .contains("stvv-dev")
                .as("the documents of an anchored record arrive in full")
                .contains("POST /api/v1/pipelines");

        // One anchor. The task list is the generic inverse relation, not a special case, and the
        // labels come back as anchors so the next call can be typed straight from the answer.
        String projectOnly = callTool("rekall_context", Map.of("anchors", "progetti:STVV"));
        assertThat(projectOnly)
                .as("an alias resolves the entity part just like the physical name")
                .contains("STVV")
                .contains("`task:code-validator-main-workflow`")
                .as("an inverse listing carries labels only, never the referencing bodies")
                .doesNotContain("POST /api/v1/pipelines");

        // The positional form, valid because "STVV" matches exactly one record across every entity.
        assertThat(callTool("rekall_context", Map.of("anchors", "STVV"))).contains("STVV");

        assertThat(callTool("rekall_context", Map.of("anchors", "task:does-not-exist")))
                .contains("No task matches");
        assertThat(callTool("rekall_context", Map.of("anchors", "nonsense:STVV")))
                .as("a wrong entity part has to name the ones that exist")
                .contains("No entity matches 'nonsense'")
                .contains("project");
    }

    @Test
    @DisplayName("the MCP role cannot write, and that is enforced by PostgreSQL")
    void readerRoleCannotWrite() {
        createTable("project", "Project", "Projects", "Tutti i progetti", List.of());
        String nameField = addField(lastTableId(), "name", "Name", "Nome del progetto", "TEXT");
        assertThat(nameField).isNotBlank();
        post("/api/meta/apply", Map.of());

        Integer refused = jdbc.queryForObject(
                """
                SELECT CASE WHEN has_table_privilege('rekall_reader', 'rekall_data.project', 'INSERT')
                            THEN 1 ELSE 0 END
                """,
                Integer.class);

        assertThat(refused).as("rekall_reader must hold SELECT and nothing else").isZero();
    }

    @Test
    @DisplayName("deleting a referenced record is refused as a conflict, not as a server error")
    void deletingReferencedRecordIsAConflict() {
        String environmentId = createTable("environment", "Environment", "Environments",
                "Configurazioni di cluster", List.of());
        String labelField = addField(environmentId, "label", "Label", "Nome dell'ambiente", "TEXT");
        setDisplayField(environmentId, "environment", "Environment", "Environments",
                "Configurazioni di cluster", List.of(), labelField);

        String taskId = createTable("task", "Task", "Tasks", "Attivita", List.of());
        String taskEnvironmentField =
                addField(taskId, "environment_id", "Environment", "L'ambiente su cui gira", "REFERENCE");
        createRelation(taskId, environmentId, taskEnvironmentField, "L'ambiente su cui gira");
        post("/api/meta/apply", Map.of());

        String environmentRecord = createRecord("environment", Map.of("label", "kmaster14"));
        createRecord("task", Map.of("environment_id", environmentRecord));

        ResponseEntity<Map> response = rest.delete()
                .uri("/api/data/environment/" + environmentRecord)
                .retrieve()
                .toEntity(Map.class);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CONFLICT);
        assertThat(String.valueOf(response.getBody().get("detail")))
                .contains("still references this record");
    }

    @Test
    @DisplayName("tools/list advertises exactly the five read-only tools")
    void toolsAreAdvertised() {
        Map<?, ?> response = rpc("tools/list", Map.of());
        Map<?, ?> result = (Map<?, ?>) response.get("result");
        List<?> tools = (List<?>) result.get("tools");

        assertThat(tools).hasSize(2);
        List<String> names =
                tools.stream().map(tool -> String.valueOf(((Map<?, ?>) tool).get("name"))).toList();
        assertThat(names).containsExactlyInAnyOrder("rekall_schema", "rekall_context");
    }

    @Test
    @DisplayName("initialize answers with the protocol version and the tools capability")
    void initializeHandshake() {
        Map<?, ?> result = (Map<?, ?>) rpc("initialize", Map.of()).get("result");

        Map<String, Object> capabilities = asMap(result.get("capabilities"));
        Map<String, Object> serverInfo = asMap(result.get("serverInfo"));

        assertThat(result.get("protocolVersion")).isEqualTo("2024-11-05");
        assertThat(capabilities).containsKey("tools");
        assertThat(serverInfo).containsEntry("name", "rekall");
    }

    // ------------------------------------------------------------------ helpers

    @SuppressWarnings("unchecked")
    private Map<String, Object> asMap(Object value) {
        return (Map<String, Object>) value;
    }

    private ResponseEntity<Map> post(String path, Object body) {
        return rest.post().uri(path).body(body).retrieve().toEntity(Map.class);
    }

    private int tableCount() {
        Integer count = jdbc.queryForObject(
                "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'rekall_data'", Integer.class);
        return count == null ? 0 : count;
    }

    private String createTable(
            String physicalName, String label, String plural, String description, List<String> aliases) {
        ResponseEntity<Map> response = post(
                "/api/meta/tables",
                Map.of(
                        "physicalName", physicalName,
                        "label", label,
                        "labelPlural", plural,
                        "description", description,
                        "aliases", aliases));
        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
        return String.valueOf(response.getBody().get("id"));
    }

    private String lastTableId() {
        return jdbc.queryForObject("SELECT id FROM rekall_meta.meta_table LIMIT 1", UUID.class).toString();
    }

    private String addField(String tableId, String column, String label, String description, String type) {
        ResponseEntity<Map> response = post(
                "/api/meta/tables/" + tableId + "/fields",
                Map.of(
                        "columnName", column,
                        "label", label,
                        "description", description,
                        "type", type,
                        "nullable", true));
        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
        Object id = response.getBody().get("id");
        assertThat(id).as("field '%s' must come back with a generated id", column).isNotNull();
        return String.valueOf(id);
    }

    private void addEnumField(String tableId, String column, String label, String description, List<String> values) {
        ResponseEntity<Map> response = post(
                "/api/meta/tables/" + tableId + "/fields",
                Map.of(
                        "columnName", column,
                        "label", label,
                        "description", description,
                        "type", "ENUM",
                        "nullable", true,
                        "length", 50,
                        "enumValues", values));
        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
    }

    private void setDisplayField(
            String tableId,
            String physicalName,
            String label,
            String plural,
            String description,
            List<String> aliases,
            String displayFieldId) {
        rest.put()
                .uri("/api/meta/tables/" + tableId)
                .body(Map.of(
                        "label", label,
                        "labelPlural", plural,
                        "description", description,
                        "aliases", aliases,
                        "displayFieldId", displayFieldId))
                .retrieve()
                .toBodilessEntity();
    }

    private void createRelation(String sourceTableId, String targetTableId, String sourceFieldId, String description) {
        ResponseEntity<Map> response = post(
                "/api/meta/relations",
                Map.of(
                        "sourceTableId", sourceTableId,
                        "targetTableId", targetTableId,
                        "kind", "MANY_TO_ONE",
                        "sourceFieldId", sourceFieldId,
                        "onDelete", "RESTRICT",
                        "description", description));
        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
    }

    private String createRecord(String entity, Map<String, Object> values) {
        ResponseEntity<Map> response = post("/api/data/" + entity, values);
        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
        return String.valueOf(response.getBody().get("id"));
    }

    private String callTool(String name, Map<String, Object> arguments) {
        Map<?, ?> response = rpc("tools/call", Map.of("name", name, "arguments", arguments));
        Map<?, ?> result = (Map<?, ?>) response.get("result");
        List<?> content = (List<?>) result.get("content");
        return String.valueOf(((Map<?, ?>) content.getFirst()).get("text"));
    }

    private Map<?, ?> rpc(String method, Map<String, Object> params) {
        ResponseEntity<Map> response =
                post("/mcp", Map.of("jsonrpc", "2.0", "id", 1, "method", method, "params", params));
        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        return response.getBody();
    }
}
