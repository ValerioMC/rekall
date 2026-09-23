package dev.rekall.claude;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.PosixFilePermissions;
import java.util.Map;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/** The environment a terminal's {@code claude} runs with, and where it is looked for. */
class ClaudeCliTest {

    @TempDir
    Path directory;

    private static ClaudeCli cliWith(String override, Map<String, String> loginEnvironment) {
        LoginShellEnvironment loginShell = mock(LoginShellEnvironment.class);
        when(loginShell.current()).thenReturn(loginEnvironment);
        return new ClaudeCli(override, loginShell);
    }

    @Test
    @DisplayName("the login shell's variables reach the terminal, its PATH first")
    void carriesTheLoginShellEnvironment() {
        ClaudeCli cli = cliWith("", Map.of(
                "PATH", "/home/me/.cargo/bin:/usr/bin:/bin",
                "RUSTUP_HOME", "/home/me/.rustup",
                "SSH_AUTH_SOCK", "/tmp/agent.sock"));

        Map<String, String> environment = cli.environment();

        assertThat(environment).containsEntry("RUSTUP_HOME", "/home/me/.rustup");
        assertThat(environment).containsEntry("SSH_AUTH_SOCK", "/tmp/agent.sock");
        assertThat(environment.get("PATH"))
                .isEqualTo("/home/me/.cargo/bin:/usr/bin:/bin:/opt/homebrew/bin:/usr/local/bin");
    }

    @Test
    @DisplayName("an enclosing Claude Code session's markers are not passed on")
    void dropsSessionMarkers() {
        ClaudeCli cli = cliWith("", Map.of("PATH", "/usr/bin", "CLAUDECODE", "1", "CLAUDE_CODE_ENTRYPOINT", "cli"));

        assertThat(cli.environment()).doesNotContainKeys("CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT");
    }

    @Test
    @DisplayName("with no PATH at all, the usual install directories stand in")
    void fallsBackToKnownDirectories() {
        ClaudeCli cli = cliWith("", Map.of());

        assertThat(cli.environment().get("PATH")).isEqualTo("/opt/homebrew/bin:/usr/local/bin:/usr/bin");
        assertThat(cli.environment()).containsKey("HOME");
    }

    @Test
    @DisplayName("an override that is executable wins; one that is not finds nothing")
    void honoursTheOverride() throws IOException {
        Path binary = directory.resolve("claude");
        Files.writeString(binary, "#!/bin/sh\n");
        Files.setPosixFilePermissions(binary, PosixFilePermissions.fromString("rwx------"));

        assertThat(cliWith(binary.toString(), Map.of()).locate()).contains(binary);
        assertThat(cliWith(directory.resolve("missing").toString(), Map.of()).locate()).isEmpty();
    }
}
