package dev.rekall.bootstrap;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.test.web.server.LocalServerPort;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.client.RestClient;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The settings endpoints, exercised over HTTP the way the Settings panel calls them.
 *
 * <p>{@code rekall.home} is pinned by the surefire configuration in {@code pom.xml} to a
 * build-local directory, never the developer's real {@code ~/.rekall}. Each test deletes
 * {@code config.json} first, so the registry always starts empty regardless of what an earlier
 * test class's context boot (which also runs {@link DatabaseLocationEnvironmentPostProcessor})
 * left behind there.
 */
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class SettingsControllerTest {

    @LocalServerPort
    private int port;

    private RestClient rest;

    @BeforeEach
    void setUp() throws IOException {
        rest = RestClient.builder()
                .baseUrl("http://localhost:" + port)
                .defaultStatusHandler(status -> true, (request, response) -> { })
                .build();
        Files.deleteIfExists(configFile());
    }

    @Test
    @DisplayName("with nothing registered, status is SETUP_NEEDED")
    void statusWithNoRegistry() {
        Map<?, ?> body = get("/api/settings/databases");

        assertThat(body.get("status")).isEqualTo("SETUP_NEEDED");
        assertThat((List<?>) body.get("databases")).isEmpty();
    }

    @Test
    @DisplayName("adding a folder that does not exist is rejected, and nothing is created")
    void addingAMissingFolderIsRejected() throws IOException {
        Path missing = Files.createTempDirectory("rekall-settings-test").resolve("does-not-exist");

        ResponseEntity<Map> response = rest.post().uri("/api/settings/databases")
                .body(Map.of("path", missing.toString()))
                .retrieve()
                .toEntity(Map.class);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
        assertThat(String.valueOf(response.getBody().get("detail"))).contains("does not exist");
        assertThat(Files.exists(missing)).isFalse();
    }

    @Test
    @DisplayName("adding an existing empty folder registers and activates it as a new database")
    void addingAnEmptyFolderCreatesANewDatabase() throws IOException {
        Path folder = Files.createTempDirectory("rekall-settings-test");

        ResponseEntity<Map> response = rest.post().uri("/api/settings/databases")
                .body(Map.of("path", folder.toString()))
                .retrieve()
                .toEntity(Map.class);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
        assertThat(response.getBody().get("mode")).isEqualTo("created");

        Map<?, ?> status = get("/api/settings/databases");
        assertThat((List<?>) status.get("databases")).hasSize(1);
        assertThat(((Map<?, ?>) status.get("active")).get("active")).isEqualTo(true);
    }

    @Test
    @DisplayName("adding a folder that already has a database opens it instead of creating one")
    void addingAFolderWithAnExistingDatabaseOpensIt() throws IOException {
        Path folder = Files.createTempDirectory("rekall-settings-test");
        Files.createFile(folder.resolve("rekall.mv.db"));

        ResponseEntity<Map> response = rest.post().uri("/api/settings/databases")
                .body(Map.of("path", folder.toString()))
                .retrieve()
                .toEntity(Map.class);

        assertThat(response.getBody().get("mode")).isEqualTo("opened");
    }

    @Test
    @DisplayName("the active database cannot be forgotten")
    void cannotForgetTheActiveDatabase() throws IOException {
        Path folder = Files.createTempDirectory("rekall-settings-test");
        String id = addAndGetId(folder);

        ResponseEntity<Map> response = rest.delete().uri("/api/settings/databases/" + id)
                .retrieve()
                .toEntity(Map.class);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CONFLICT);
    }

    @Test
    @DisplayName("renaming changes the label without touching which database is active")
    void renamingChangesTheLabel() throws IOException {
        Path folder = Files.createTempDirectory("rekall-settings-test");
        String id = addAndGetId(folder);

        Map<?, ?> renamed = rest.patch().uri("/api/settings/databases/" + id)
                .body(Map.of("label", "Renamed"))
                .retrieve()
                .body(Map.class);

        assertThat(renamed.get("label")).isEqualTo("Renamed");
        assertThat(renamed.get("active")).isEqualTo(true);
    }

    @Test
    @DisplayName("a blank label is rejected")
    void aBlankLabelIsRejected() throws IOException {
        Path folder = Files.createTempDirectory("rekall-settings-test");
        String id = addAndGetId(folder);

        ResponseEntity<Map> response = rest.patch().uri("/api/settings/databases/" + id)
                .body(Map.of("label", "  "))
                .retrieve()
                .toEntity(Map.class);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
    }

    @Test
    @DisplayName("the live check endpoint never registers anything")
    void checkIsReadOnly() throws IOException {
        Path folder = Files.createTempDirectory("rekall-settings-test");

        Map<?, ?> check = get("/api/settings/databases/check?path=" + folder);

        assertThat(check.get("usable")).isEqualTo(true);
        assertThat((List<?>) get("/api/settings/databases").get("databases")).isEmpty();
    }

    // ------------------------------------------------------------------ helpers

    private String addAndGetId(Path folder) {
        Map<?, ?> added = post("/api/settings/databases", Map.of("path", folder.toString()));
        return (String) ((Map<?, ?>) added.get("entry")).get("id");
    }

    private Path configFile() {
        return Path.of(System.getProperty("rekall.home"), "config.json");
    }

    @SuppressWarnings("unchecked")
    private Map<?, ?> get(String path) {
        return rest.get().uri(path).retrieve().body(Map.class);
    }

    @SuppressWarnings("unchecked")
    private Map<?, ?> post(String path, Object body) {
        return rest.post().uri(path).body(body).retrieve().body(Map.class);
    }
}
