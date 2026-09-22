package dev.rekall.backup;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.test.web.server.LocalServerPort;
import org.springframework.core.io.ByteArrayResource;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.springframework.util.LinkedMultiValueMap;
import org.springframework.util.MultiValueMap;
import org.springframework.web.client.RestClient;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Backups against a real file database in a folder of its own, since an in-memory database, which
 * every other test runs on, has none.
 */
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class BackupApiTest {

    private static final Path FOLDER = temporaryFolder();

    @DynamicPropertySource
    static void fileDatabase(DynamicPropertyRegistry registry) {
        registry.add("spring.datasource.url", () -> "jdbc:h2:file:" + FOLDER + "/rekall;DB_CLOSE_DELAY=-1");
        registry.add("rekall.backup.enabled", () -> "false");
        registry.add("rekall.backup.keep", () -> "3");
    }

    @LocalServerPort
    private int port;

    private RestClient rest() {
        return RestClient.builder()
                .baseUrl("http://localhost:" + port)
                .defaultStatusHandler(status -> true, (request, response) -> { })
                .build();
    }

    @Test
    @DisplayName("a backup taken now is an H2 database zipped in the backups folder, listed and downloadable")
    void aBackupIsTakenListedAndDownloaded() throws IOException {
        Map<?, ?> taken = rest().post().uri("/api/backups").retrieve().body(Map.class);

        String name = String.valueOf(taken.get("name"));
        assertThat(name).matches("rekall-\\d{8}-\\d{6}(-\\d+)?-manual\\.zip");
        assertThat(FOLDER.resolve("backups").resolve(name)).isRegularFile();

        Map<?, ?> status = rest().get().uri("/api/backups").retrieve().body(Map.class);
        assertThat(status.get("available")).isEqualTo(true);
        List<Object> names = ((List<?>) status.get("backups")).stream()
                .map(backup -> (Object) ((Map<?, ?>) backup).get("name"))
                .toList();
        assertThat(names).contains(name);

        byte[] zip = rest().get().uri("/api/backups/{name}", name).retrieve().body(byte[].class);
        try (ZipInputStream in = new ZipInputStream(new ByteArrayInputStream(zip))) {
            ZipEntry entry = in.getNextEntry();
            assertThat(entry.getName()).isEqualTo("rekall.mv.db");
            assertThat(new String(in.readNBytes(3), StandardCharsets.US_ASCII))
                    .as("the header the restore checks for")
                    .isEqualTo("H:2");
        }
    }

    @Test
    @DisplayName("past the configured number, the oldest backups are dropped")
    void theOldestArePruned() {
        for (int at = 0; at < 5; at++) {
            rest().post().uri("/api/backups").retrieve().toBodilessEntity();
        }

        Map<?, ?> status = rest().get().uri("/api/backups").retrieve().body(Map.class);
        assertThat((List<?>) status.get("backups")).hasSize(3);
    }

    @Test
    @DisplayName("a name that is not one the folder handed out is refused, so it can never be a path")
    void aForeignNameIsRefused() {
        assertThat(rest().get().uri("/api/backups/{name}", "..%2Frekall.mv.db").retrieve().toBodilessEntity()
                .getStatusCode().value()).isIn(400, 404);
        assertThat(rest().post().uri("/api/backups/{name}/restore", "rekall-20260101-000000-auto.zip")
                .retrieve().toBodilessEntity().getStatusCode().value()).isEqualTo(400);
    }

    @Test
    @DisplayName("an upload that is not a backup is refused, and nothing is left behind")
    void aBadUploadIsRefused() {
        MultiValueMap<String, Object> form = new LinkedMultiValueMap<>();
        form.add("file", new ByteArrayResource("not a zip".getBytes(StandardCharsets.UTF_8)) {
            @Override
            public String getFilename() {
                return "backup.zip";
            }
        });

        ResponseEntity<Map> answer = rest().post().uri("/api/backups/restore")
                .contentType(MediaType.MULTIPART_FORM_DATA)
                .body(form)
                .retrieve()
                .toEntity(Map.class);

        assertThat(answer.getStatusCode().value()).as("the file is judged before anything else").isEqualTo(400);
        assertThat(FOLDER.resolve("rekall.mv.db.restoring")).doesNotExist();
        assertThat(uploadedBackups()).isEmpty();
    }

    private static List<Path> uploadedBackups() {
        Path backups = FOLDER.resolve("backups");
        if (!Files.isDirectory(backups)) {
            return List.of();
        }
        try (var files = Files.list(backups)) {
            return files.filter(file -> file.getFileName().toString().endsWith("-uploaded.zip")).toList();
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    private static Path temporaryFolder() {
        try {
            return Files.createTempDirectory("rekall-backup-test");
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }
}
