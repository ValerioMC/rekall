package dev.rekall;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.test.web.server.LocalServerPort;
import org.springframework.http.HttpStatus;
import org.springframework.http.HttpStatusCode;
import org.springframework.http.ResponseEntity;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.springframework.web.client.RestClient;
import org.testcontainers.containers.PostgreSQLContainer;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Every endpoint the frontend calls, in the order the frontend calls it.
 *
 * <p>Written after a bug that no other test could have caught: the API maps entities to DTOs
 * once the service transaction has already closed, so an association left lazy fails at
 * serialisation time. Tests that call services directly never see it, and tests that happen to
 * exercise only some endpoints only catch some of it. This walks the whole client surface, so
 * a screen cannot break without a test breaking first.
 */
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class UiEndpointCoverageTest {

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
    void setUp() {
        rest = RestClient.builder()
                .baseUrl("http://localhost:" + port)
                .defaultStatusHandler(status -> true, (request, response) -> { })
                .build();

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
    @DisplayName("the schema designer screens: list, open, edit, add and remove fields and relations")
    void schemaDesignerEndpoints() {
        // SchemaView on load
        assertOkList(getList("/api/meta/tables"), "list entities");
        assertOkList(getList("/api/meta/relations"), "list relations");
        assertOk(get("/api/meta/plan"), "plan badge in the sidebar");

        // SchemaView: create
        String projectId = id(assertCreated(
                post("/api/meta/tables", Map.of(
                        "physicalName", "project", "label", "Project", "labelPlural", "Projects",
                        "description", "Tutti i progetti", "aliases", List.of("progetti"))),
                "create entity"));

        // EntityView on load. This is the call that used to fail with LazyInitializationException.
        ResponseEntity<Map> detail = get("/api/meta/tables/" + projectId);
        assertOk(detail, "open an entity");
        assertThat((List<?>) detail.getBody().get("fields"))
                .as("the field list has to survive leaving the transaction")
                .isNotNull();

        // EntityView: add fields of every kind the form can produce
        String nameFieldId = id(assertCreated(
                post("/api/meta/tables/" + projectId + "/fields", Map.of(
                        "columnName", "name", "label", "Name", "description", "Nome del progetto",
                        "type", "TEXT", "nullable", false)),
                "add a text field"));
        assertCreated(
                post("/api/meta/tables/" + projectId + "/fields", Map.of(
                        "columnName", "status", "label", "Status", "description", "Stato",
                        "type", "ENUM", "nullable", true, "length", 50,
                        "enumValues", List.of("active", "done"))),
                "add an enum field");
        assertCreated(
                post("/api/meta/tables/" + projectId + "/fields", Map.of(
                        "columnName", "labels", "label", "Labels", "description", "Etichette",
                        "type", "TAGS", "nullable", true)),
                "add a tags field");
        String scratchFieldId = id(assertCreated(
                post("/api/meta/tables/" + projectId + "/fields", Map.of(
                        "columnName", "scratch", "label", "Scratch", "description", "Da rimuovere",
                        "type", "LONG_TEXT", "nullable", true)),
                "add a field that will be removed"));

        // EntityView: set the identifying field, which is a PUT on the entity
        assertOk(
                put("/api/meta/tables/" + projectId, Map.of(
                        "label", "Project", "labelPlural", "Projects", "description", "Tutti i progetti",
                        "aliases", List.of("progetti"), "displayFieldId", nameFieldId)),
                "set the identifying field");

        // EntityView: edit and remove a field
        assertOk(
                put("/api/meta/fields/" + scratchFieldId, Map.of(
                        "label", "Scratch renamed", "description", "Ancora da rimuovere",
                        "type", "LONG_TEXT", "nullable", true)),
                "edit a field");
        assertNoContent(delete("/api/meta/fields/" + scratchFieldId), "remove a field");

        // EntityView: relations
        String environmentId = id(assertCreated(
                post("/api/meta/tables", Map.of(
                        "physicalName", "environment", "label", "Environment", "labelPlural", "Environments",
                        "description", "Ambienti", "aliases", List.of())),
                "create a second entity"));
        String referenceFieldId = id(assertCreated(
                post("/api/meta/tables/" + projectId + "/fields", Map.of(
                        "columnName", "environment_id", "label", "Environment",
                        "description", "L'ambiente su cui gira", "type", "REFERENCE", "nullable", true)),
                "add a reference field"));
        String relationId = id(assertCreated(
                post("/api/meta/relations", Map.of(
                        "sourceTableId", projectId, "targetTableId", environmentId,
                        "kind", "MANY_TO_ONE", "sourceFieldId", referenceFieldId,
                        "onDelete", "RESTRICT", "description", "L'ambiente su cui gira")),
                "create a relation"));
        assertNoContent(delete("/api/meta/relations/" + relationId), "remove a relation");

        // SchemaView: delete an entity
        assertNoContent(delete("/api/meta/tables/" + environmentId), "delete an entity");
    }

    @Test
    @DisplayName("the plan screen: preview, answer a question, apply")
    void planEndpoints() {
        String projectId = id(post("/api/meta/tables", Map.of(
                "physicalName", "project", "label", "Project", "labelPlural", "Projects",
                "description", "Tutti i progetti", "aliases", List.of())).getBody());
        post("/api/meta/tables/" + projectId + "/fields", Map.of(
                "columnName", "name", "label", "Name", "description", "Nome", "type", "TEXT", "nullable", true));

        assertOk(get("/api/meta/plan"), "plan on load");
        assertOk(post("/api/meta/plan", Map.of("backfillDefaults", Map.of(), "confirmedDrops", List.of())),
                "re-plan after an answer");
        assertOk(post("/api/meta/apply", Map.of("backfillDefaults", Map.of(), "confirmedDrops", List.of())),
                "apply");

        // Applying with nothing to do is refused, and the UI must see a conflict rather than a 500.
        ResponseEntity<Map> again =
                post("/api/meta/apply", Map.of("backfillDefaults", Map.of(), "confirmedDrops", List.of()));
        assertThat(again.getStatusCode()).isEqualTo(HttpStatus.CONFLICT);
        assertThat(String.valueOf(again.getBody().get("detail"))).contains("nothing to apply");
    }

    @Test
    @DisplayName("the data browser and record screens, including documents and search")
    void dataAndDocumentEndpoints() {
        String projectId = id(post("/api/meta/tables", Map.of(
                "physicalName", "project", "label", "Project", "labelPlural", "Projects",
                "description", "Tutti i progetti", "aliases", List.of())).getBody());
        String nameFieldId = id(post("/api/meta/tables/" + projectId + "/fields", Map.of(
                "columnName", "name", "label", "Name", "description", "Nome",
                "type", "TEXT", "nullable", true)).getBody());
        post("/api/meta/tables/" + projectId + "/fields", Map.of(
                "columnName", "labels", "label", "Labels", "description", "Etichette",
                "type", "TAGS", "nullable", true));
        put("/api/meta/tables/" + projectId, Map.of(
                "label", "Project", "labelPlural", "Projects", "description", "Tutti i progetti",
                "aliases", List.of(), "displayFieldId", nameFieldId));
        post("/api/meta/apply", Map.of("backfillDefaults", Map.of(), "confirmedDrops", List.of()));

        // DataView on load, and its search box
        assertOk(get("/api/data/project?search=&limit=50&offset=0"), "list records");
        assertOk(get("/api/data/project?search=STV&limit=50&offset=0"), "search records");

        String recordId = id(assertCreated(
                post("/api/data/project", Map.of("name", "STVV", "labels", List.of("esa", "backend"))),
                "create a record"));

        // RecordView on load
        assertOk(get("/api/data/project/" + recordId), "open a record");
        assertOkList(getList("/api/documents?entity=project&recordId=" + recordId), "list its documents");

        assertOk(put("/api/data/project/" + recordId, Map.of("name", "STVV-A")), "save a record");

        String documentId = id(assertCreated(
                post("/api/documents", Map.of(
                        "entityName", "project", "recordId", recordId, "title", "CONTEXT.md",
                        "kind", "context", "bodyMarkdown", "# Contesto\n\nCluster kmaster14.")),
                "add a document"));
        assertOk(get("/api/documents/" + documentId), "open a document");
        assertOk(put("/api/documents/" + documentId, Map.of(
                "title", "CONTEXT.md", "kind", "context", "bodyMarkdown", "# Contesto aggiornato")),
                "save a document");

        // SearchView
        assertOkList(getList("/api/documents/search?query=kmaster14"), "search documents");

        assertNoContent(delete("/api/documents/" + documentId), "delete a document");
        assertNoContent(delete("/api/data/project/" + recordId), "delete a record");
    }

    @Test
    @DisplayName("the maintenance endpoints import a folder tree and export it back")
    void maintenanceEndpoints() throws Exception {
        String projectId = id(post("/api/meta/tables", Map.of(
                "physicalName", "project", "label", "Project", "labelPlural", "Projects",
                "description", "Tutti i progetti", "aliases", List.of())).getBody());
        String nameFieldId = id(post("/api/meta/tables/" + projectId + "/fields", Map.of(
                "columnName", "name", "label", "Name", "description", "Nome",
                "type", "TEXT", "nullable", true)).getBody());
        put("/api/meta/tables/" + projectId, Map.of(
                "label", "Project", "labelPlural", "Projects", "description", "Tutti i progetti",
                "aliases", List.of(), "displayFieldId", nameFieldId));
        post("/api/meta/apply", Map.of("backfillDefaults", Map.of(), "confirmedDrops", List.of()));

        // A miniature of the folder tree this application was built to replace.
        Path root = Files.createTempDirectory("rekall-import");
        Path project = Files.createDirectories(root.resolve("stvv"));
        Files.writeString(project.resolve("CONTEXT.md"), "# STVV\n\nCoordinate del progetto.");

        ResponseEntity<Map> imported = post("/api/maintenance/import", Map.of("path", root.toString()));
        assertOk(imported, "import a folder");
        assertThat(imported.getBody().get("projectsCreated")).isEqualTo(1);
        assertThat(imported.getBody().get("documentsCreated")).isEqualTo(1);

        Path exportTarget = Files.createTempDirectory("rekall-export");
        ResponseEntity<Map> exported = post("/api/maintenance/export", Map.of("path", exportTarget.toString()));
        assertOk(exported, "export to a folder");
        assertThat(exported.getBody().get("filesWritten")).isEqualTo(1);
        assertThat(Files.exists(exportTarget.resolve("project/stvv/CONTEXT.md"))).isTrue();
    }

    @Test
    @DisplayName("mistakes the UI can produce come back as 4xx with a readable message, never as 500")
    void errorsAreReadable() {
        assertThat(get("/api/meta/tables/" + java.util.UUID.randomUUID()).getStatusCode())
                .isEqualTo(HttpStatus.NOT_FOUND);
        assertThat(get("/api/data/does_not_exist?search=&limit=50&offset=0").getStatusCode())
                .isEqualTo(HttpStatus.NOT_FOUND);

        ResponseEntity<Map> reserved = post("/api/meta/tables", Map.of(
                "physicalName", "select", "label", "Select", "labelPlural", "Selects",
                "description", "Nome riservato", "aliases", List.of()));
        assertThat(reserved.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
        assertThat(String.valueOf(reserved.getBody().get("detail"))).contains("reserved");

        ResponseEntity<Map> blank = post("/api/meta/tables", Map.of(
                "physicalName", "note", "label", "Note", "labelPlural", "Notes",
                "description", "", "aliases", List.of()));
        assertThat(blank.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);

        String projectId = id(post("/api/meta/tables", Map.of(
                "physicalName", "project", "label", "Project", "labelPlural", "Projects",
                "description", "Tutti i progetti", "aliases", List.of())).getBody());
        ResponseEntity<Map> duplicate = post("/api/meta/tables", Map.of(
                "physicalName", "project", "label", "Project", "labelPlural", "Projects",
                "description", "Duplicato", "aliases", List.of()));
        assertThat(duplicate.getStatusCode()).isEqualTo(HttpStatus.CONFLICT);

        post("/api/meta/tables/" + projectId + "/fields", Map.of(
                "columnName", "name", "label", "Name", "description", "Nome",
                "type", "TEXT", "nullable", true));
        post("/api/meta/apply", Map.of("backfillDefaults", Map.of(), "confirmedDrops", List.of()));

        ResponseEntity<Map> unknownField = post("/api/data/project", Map.of("nome", "STVV"));
        assertThat(unknownField.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
        assertThat(String.valueOf(unknownField.getBody().get("detail"))).contains("has no field");
    }

    // ------------------------------------------------------------------ helpers

    private ResponseEntity<Map> get(String path) {
        return rest.get().uri(path).retrieve().toEntity(Map.class);
    }

    /** Several endpoints answer with a JSON array rather than an object. */
    private ResponseEntity<List> getList(String path) {
        return rest.get().uri(path).retrieve().toEntity(List.class);
    }

    private ResponseEntity<Map> post(String path, Object body) {
        return rest.post().uri(path).body(body).retrieve().toEntity(Map.class);
    }

    private ResponseEntity<Map> put(String path, Object body) {
        return rest.put().uri(path).body(body).retrieve().toEntity(Map.class);
    }

    private ResponseEntity<Map> delete(String path) {
        return rest.delete().uri(path).retrieve().toEntity(Map.class);
    }

    private ResponseEntity<Map> assertOk(ResponseEntity<Map> response, String what) {
        assertStatus(response, HttpStatus.OK, what);
        return response;
    }

    private void assertOkList(ResponseEntity<List> response, String what) {
        assertThat(response.getStatusCode()).as("%s", what).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).as("%s returned no body", what).isNotNull();
    }

    private Map<?, ?> assertCreated(ResponseEntity<Map> response, String what) {
        assertStatus(response, HttpStatus.CREATED, what);
        return response.getBody();
    }

    private void assertNoContent(ResponseEntity<Map> response, String what) {
        assertStatus(response, HttpStatus.NO_CONTENT, what);
    }

    /** Failures name the screen action, so a red test says which button broke. */
    private void assertStatus(ResponseEntity<Map> response, HttpStatusCode expected, String what) {
        assertThat(response.getStatusCode())
                .as("%s: %s", what, response.getBody() == null ? "" : response.getBody().get("detail"))
                .isEqualTo(expected);
    }

    private String id(Map<?, ?> body) {
        return String.valueOf(body.get("id"));
    }
}
