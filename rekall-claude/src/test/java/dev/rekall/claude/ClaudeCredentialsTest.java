package dev.rekall.claude;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The non-keychain half of the token read: {@code ~/.claude/.credentials.json}. The keychain path
 * is macOS-only and environment-bound, so it is left to the machine it runs on.
 */
class ClaudeCredentialsTest {

    @TempDir
    Path home;

    private ClaudeCredentials credentials() {
        return new ClaudeCredentials(home, false);
    }

    @Test
    @DisplayName("reads the access token out of the credentials file")
    void readsFromFile() throws IOException {
        writeCredentials("{\"claudeAiOauth\":{\"accessToken\":\"sk-ant-oat01-abc\"}}");

        assertThat(credentials().accessToken()).contains("sk-ant-oat01-abc");
    }

    @Test
    @DisplayName("no file means no token, not an error")
    void missingFile() {
        assertThat(credentials().accessToken()).isEmpty();
    }

    @Test
    @DisplayName("a file that is not the expected shape yields no token")
    void wrongShape() throws IOException {
        writeCredentials("{\"something\":\"else\"}");

        assertThat(credentials().accessToken()).isEmpty();
    }

    @Test
    @DisplayName("a blank token is treated as absent")
    void blankToken() throws IOException {
        writeCredentials("{\"claudeAiOauth\":{\"accessToken\":\"  \"}}");

        assertThat(credentials().accessToken()).isEmpty();
    }

    private void writeCredentials(String json) throws IOException {
        Path dir = Files.createDirectories(home.resolve(".claude"));
        Files.writeString(dir.resolve(".credentials.json"), json);
    }
}
