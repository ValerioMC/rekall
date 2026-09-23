package dev.rekall.claude;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.PosixFilePermissions;
import java.util.Map;
import java.util.Optional;

import static org.assertj.core.api.Assertions.assertThat;

/** Asking a login shell for its environment, and every way the shell can fail to answer. */
class LoginShellEnvironmentTest {

    @TempDir
    Path directory;

    /** A stand-in for {@code $SHELL}: runs {@code body}, then the {@code -c} script it was handed as its fourth argument. */
    private Path fakeShell(String body) throws IOException {
        Path shell = directory.resolve("fake-shell");
        Files.writeString(shell, "#!/bin/sh\n" + body + "\neval \"$4\"\n");
        Files.setPosixFilePermissions(shell, PosixFilePermissions.fromString("rwx------"));
        return shell;
    }

    @Test
    @DisplayName("what the profile exports comes back, and what it prints around the markers does not")
    void resolvesWhatTheProfileExports() throws IOException {
        Path shell = fakeShell("""
                export PATH="/home/me/.cargo/bin:$PATH"
                export RUSTUP_HOME=/home/me/.rustup
                echo 'Welcome back, last login yesterday'
                """);

        Map<String, String> environment = new LoginShellEnvironment(shell.toString(), 5).resolve().orElseThrow();

        assertThat(environment.get("PATH")).startsWith("/home/me/.cargo/bin:");
        assertThat(environment).containsEntry("RUSTUP_HOME", "/home/me/.rustup");
        assertThat(environment.keySet()).noneMatch(key -> key.contains("Welcome"));
    }

    @Test
    @DisplayName("the resolving shell's own variables and the resolving flag are left out")
    void dropsTheShellsOwnVariables() throws IOException {
        Path shell = fakeShell("[ -n \"$REKALL_RESOLVING_ENVIRONMENT\" ] && export SAW_FLAG=yes");

        Map<String, String> environment = new LoginShellEnvironment(shell.toString(), 5).resolve().orElseThrow();

        assertThat(environment).containsEntry("SAW_FLAG", "yes");
        assertThat(environment).doesNotContainKeys("_", "SHLVL", "PWD", "OLDPWD", LoginShellEnvironment.RESOLVING_FLAG);
    }

    @Test
    @DisplayName("a value holding '=' or a newline survives intact")
    void keepsAwkwardValues() throws IOException {
        Path shell = fakeShell("export ODD='a=b\nsecond line'");

        Map<String, String> environment = new LoginShellEnvironment(shell.toString(), 5).resolve().orElseThrow();

        assertThat(environment).containsEntry("ODD", "a=b\nsecond line");
    }

    @Test
    @DisplayName("a shell that never finishes is abandoned at the timeout")
    void abandonsAHangingShell() throws IOException {
        Path shell = fakeShell("sleep 30");

        long started = System.nanoTime();
        Optional<Map<String, String>> environment = new LoginShellEnvironment(shell.toString(), 1).resolve();

        assertThat(environment).isEmpty();
        assertThat(System.nanoTime() - started).isLessThan(10_000_000_000L);
    }

    @Test
    @DisplayName("a shell that exits before printing anything, or is not there, gives nothing")
    void givesNothingWithoutAnAnswer() throws IOException {
        assertThat(new LoginShellEnvironment(fakeShell("exit 1").toString(), 5).resolve()).isEmpty();
        assertThat(new LoginShellEnvironment(directory.resolve("missing").toString(), 5).resolve()).isEmpty();
    }

    @Test
    @DisplayName("with no answer from the shell, current() falls back to this process's environment")
    void fallsBackToTheProcessEnvironment() throws IOException {
        LoginShellEnvironment loginShell = new LoginShellEnvironment(fakeShell("exit 1").toString(), 5);

        assertThat(loginShell.current()).isEqualTo(System.getenv());
    }

    @Test
    @DisplayName("current() hands back the resolved environment")
    void currentReturnsTheResolvedEnvironment() throws IOException {
        LoginShellEnvironment loginShell = new LoginShellEnvironment(fakeShell("export FROM_PROFILE=1").toString(), 5);

        assertThat(loginShell.current()).containsEntry("FROM_PROFILE", "1");
    }

    @Test
    @DisplayName("parse needs both markers")
    void parseNeedsBothMarkers() {
        assertThat(LoginShellEnvironment.parse("noise only", "MARK")).isEmpty();
        assertThat(LoginShellEnvironment.parse("MARKA=1\0", "MARK")).isEmpty();
        assertThat(LoginShellEnvironment.parse("noise MARKA=1\0B=2\0MARK noise", "MARK"))
                .contains(Map.of("A", "1", "B", "2"));
    }
}
