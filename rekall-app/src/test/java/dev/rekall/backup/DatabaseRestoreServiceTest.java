package dev.rekall.backup;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

class DatabaseRestoreServiceTest {

    @TempDir
    Path folder;

    @Test
    @DisplayName("the database file is taken out of the zip under its own name")
    void theDatabaseFileIsExtracted() throws IOException {
        Path zip = zip(Map.of("rekall.mv.db", "H:2,block:0 the rest"));
        Path target = folder.resolve("rekall.mv.db.restoring");

        DatabaseRestoreService.extractDatabase(zip, "rekall.mv.db", target);

        assertThat(Files.readString(target)).startsWith("H:2");
    }

    @Test
    @DisplayName("a backup of a database with another name is accepted when it is the only one in the zip")
    void aSingleDatabaseUnderAnotherNameIsAccepted() throws IOException {
        Path zip = zip(Map.of("other.mv.db", "H:2 data"));
        Path target = folder.resolve("rekall.mv.db.restoring");

        DatabaseRestoreService.extractDatabase(zip, "rekall.mv.db", target);

        assertThat(target).exists();
    }

    @Test
    @DisplayName("a zip with no database, two databases, or a file that is not H2 is refused and leaves nothing behind")
    void anythingElseIsRefused() throws IOException {
        Path target = folder.resolve("rekall.mv.db.restoring");

        assertThatThrownBy(() -> DatabaseRestoreService.extractDatabase(
                zip(Map.of("notes.md", "hello")), "rekall.mv.db", target))
                .isInstanceOf(IllegalArgumentException.class).hasMessageContaining("no H2 database file");
        assertThatThrownBy(() -> DatabaseRestoreService.extractDatabase(
                zip(Map.of("a.mv.db", "H:2", "b.mv.db", "H:2")), "rekall.mv.db", target))
                .isInstanceOf(IllegalArgumentException.class).hasMessageContaining("more than one");
        assertThatThrownBy(() -> DatabaseRestoreService.extractDatabase(
                zip(Map.of("rekall.mv.db", "PK not a database")), "rekall.mv.db", target))
                .isInstanceOf(IllegalArgumentException.class).hasMessageContaining("not an H2 database file");
        assertThat(target).doesNotExist();
    }

    @Test
    @DisplayName("a file that is not a zip at all is refused")
    void aNonZipIsRefused() throws IOException {
        Path notZip = Files.writeString(folder.resolve("x.zip"), "plain text");

        assertThatThrownBy(() -> DatabaseRestoreService.extractDatabase(
                notZip, "rekall.mv.db", folder.resolve("out")))
                .isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    @DisplayName("only a file database has a location: its folder, its base name, and a backups folder beside it")
    void theLocationIsReadFromTheUrl() {
        assertThat(DatabaseLocation.of("jdbc:h2:mem:rekall;DB_CLOSE_DELAY=-1")).isEmpty();
        DatabaseLocation location = DatabaseLocation.of("jdbc:h2:file:/data/rekall/rekall;AUTO_SERVER=TRUE").orElseThrow();
        assertThat(location.folder()).isEqualTo(Path.of("/data/rekall"));
        assertThat(location.dataFile()).isEqualTo(Path.of("/data/rekall/rekall.mv.db"));
        assertThat(location.backups()).isEqualTo(Path.of("/data/rekall/backups"));
    }

    private Path zip(Map<String, String> entries) throws IOException {
        Path zip = Files.createTempFile(folder, "backup", ".zip");
        try (OutputStream out = Files.newOutputStream(zip); ZipOutputStream writer = new ZipOutputStream(out)) {
            for (Map.Entry<String, String> entry : entries.entrySet()) {
                writer.putNextEntry(new ZipEntry(entry.getKey()));
                writer.write(entry.getValue().getBytes(StandardCharsets.UTF_8));
                writer.closeEntry();
            }
        }
        return zip;
    }
}
