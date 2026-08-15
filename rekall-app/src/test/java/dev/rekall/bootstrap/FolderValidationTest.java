package dev.rekall.bootstrap;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.assertj.core.api.Assertions.assertThat;

class FolderValidationTest {

    @TempDir
    Path tempDir;

    @Test
    void aFolderThatDoesNotExistIsReportedAsSuch() {
        FolderValidation.Result result = FolderValidation.inspect(tempDir.resolve("nowhere").toString());

        assertThat(result.exists()).isFalse();
        assertThat(result.usable()).isFalse();
    }

    @Test
    void anEmptyExistingFolderIsUsableButHasNoDatabaseYet() {
        FolderValidation.Result result = FolderValidation.inspect(tempDir.toString());

        assertThat(result.usable()).isTrue();
        assertThat(result.hasDatabase()).isFalse();
    }

    @Test
    void aFolderWithAnMvDbFileIsReportedAsAlreadyHavingADatabase() throws IOException {
        Files.createFile(tempDir.resolve("rekall.mv.db"));

        assertThat(FolderValidation.inspect(tempDir.toString()).hasDatabase()).isTrue();
    }

    @Test
    void blankInputIsNeverUsable() {
        assertThat(FolderValidation.inspect("   ").usable()).isFalse();
        assertThat(FolderValidation.inspect(null).usable()).isFalse();
    }

    @Test
    void aFileRatherThanAFolderIsNotUsable() throws IOException {
        Path file = tempDir.resolve("not-a-folder");
        Files.createFile(file);

        FolderValidation.Result result = FolderValidation.inspect(file.toString());

        assertThat(result.exists()).isTrue();
        assertThat(result.isDirectory()).isFalse();
        assertThat(result.usable()).isFalse();
    }

    @Test
    void aLeadingTildeExpandsToTheHomeDirectory() {
        FolderValidation.Result result = FolderValidation.inspect("~");

        assertThat(result.resolvedPath()).isEqualTo(Path.of(System.getProperty("user.home")).toString());
    }
}
