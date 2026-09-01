package dev.rekall.bootstrap;

import dev.rekall.api.service.ConflictException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

/**
 * The Claude Code registration, against a home directory of the test's own.
 *
 * <p>Nothing here launches the real CLI: the runner is a stand-in that records what it was asked
 * to run and, for an {@code mcp add} or {@code mcp remove}, rewrites the configuration the real
 * one would have rewritten. That is the whole contract this class has with Claude Code, so it is
 * the thing worth pinning down. {@code searchPath} is passed explicitly by every test, because
 * whether the machine running the build happens to have a {@code claude} on its PATH is not what
 * is under test.
 */
class ClaudeCodeInstallerTest {

    private static final String ENDPOINT = "http://localhost:47355/mcp";

    @TempDir
    Path home;

    private final List<List<String>> commands = new ArrayList<>();
    private final List<Path> directories = new ArrayList<>();

    /** The state {@code ~/.claude.json} is rendered from, so a removal can be simulated. */
    private String userScopeUrl;
    private final Set<Path> folderScoped = new LinkedHashSet<>();

    /** Records every command and answers success, writing nothing. Replaced where a test needs more. */
    private ClaudeCodeInstaller.CommandRunner runner = (command, environment, directory) -> {
        commands.add(command);
        directories.add(directory);
        return new ClaudeCodeInstaller.Outcome(0, "");
    };

    /** Stands in for the real CLI: what it is asked to register or remove, it registers or removes. */
    private final ClaudeCodeInstaller.CommandRunner realistic = (command, environment, directory) -> {
        commands.add(command);
        directories.add(directory);
        if (command.contains("add")) {
            userScopeUrl = command.getLast();
        } else if (command.contains("remove") && command.contains("local")) {
            folderScoped.remove(directory);
        } else if (command.contains("remove")) {
            userScopeUrl = null;
        }
        writeConfiguration();
        return new ClaudeCodeInstaller.Outcome(0, "");
    };

    @BeforeEach
    void setUp() {
        commands.clear();
        directories.clear();
        folderScoped.clear();
        userScopeUrl = null;
    }

    @Test
    @DisplayName("with no CLI and nothing registered, it reports what to run by hand")
    void nothingInstalled() {
        ClaudeCodeInstaller.Installation status = installer().status();

        assertThat(status.status()).isEqualTo(ClaudeCodeInstaller.CLI_MISSING);
        assertThat(status.registeredUrl()).isNull();
        assertThat(status.cliPath()).isNull();
        assertThat(status.commandInstalled()).isFalse();
        assertThat(status.manualCommand()).isEqualTo(
                "claude mcp add --scope user --transport http rekall " + ENDPOINT);
    }

    @Test
    @DisplayName("a CLI on this machine and no registration is NOT_CONNECTED, not a missing CLI")
    void cliPresentButUnregistered() throws IOException {
        Path cli = fakeCli();

        ClaudeCodeInstaller.Installation status = installer().status();

        assertThat(status.status()).isEqualTo(ClaudeCodeInstaller.NOT_CONNECTED);
        assertThat(status.cliPath()).isEqualTo(cli.toString());
    }

    @Test
    @DisplayName("the user-scope registration and a current command file together are CONNECTED")
    void registeredAndCurrent() throws IOException {
        fakeCli();
        registerUserScope(ENDPOINT);
        writeCommandFile(packagedCommand());

        assertThat(installer().status().status()).isEqualTo(ClaudeCodeInstaller.CONNECTED);
    }

    @Test
    @DisplayName("a registration pointing at another port is OUTDATED")
    void registeredElsewhere() throws IOException {
        fakeCli();
        registerUserScope("http://localhost:8080/mcp");
        writeCommandFile(packagedCommand());

        ClaudeCodeInstaller.Installation status = installer().status();

        assertThat(status.status()).isEqualTo(ClaudeCodeInstaller.OUTDATED);
        assertThat(status.registeredUrl()).isEqualTo("http://localhost:8080/mcp");
    }

    @Test
    @DisplayName("an older copy of the slash command is OUTDATED even when the server is right")
    void staleCommandFile() throws IOException {
        fakeCli();
        registerUserScope(ENDPOINT);
        writeCommandFile("an earlier version of the command");

        ClaudeCodeInstaller.Installation status = installer().status();

        assertThat(status.status()).isEqualTo(ClaudeCodeInstaller.OUTDATED);
        assertThat(status.commandInstalled()).isFalse();
    }

    @Test
    @DisplayName("registrations made per folder are reported, and only for the folders that carry one")
    void folderScopedRegistrationsAreReported() throws IOException {
        fakeCli();
        Path carrying = projectFolder("carrying");
        projectFolder("plain");
        registerFolderScope(carrying);

        ClaudeCodeInstaller.Installation status = installer().status();

        assertThat(status.folderScoped()).containsExactly(carrying.toString());
        assertThat(status.registeredUrl()).isNull();
        assertThat(status.status()).isEqualTo(ClaudeCodeInstaller.NOT_CONNECTED);
    }

    @Test
    @DisplayName("a folder carrying a copy of its own is not CONNECTED, right as the user-scope one is")
    void aFolderScopedCopyIsNotConnected() throws IOException {
        fakeCli();
        registerUserScope(ENDPOINT);
        writeCommandFile(packagedCommand());
        registerFolderScope(projectFolder("carrying"));

        // Inside that folder it is the copy a session finds, so this is not one registration
        // answering from everywhere, which is the only thing CONNECTED is allowed to mean.
        assertThat(installer().status().status()).isEqualTo(ClaudeCodeInstaller.OUTDATED);
    }

    @Test
    @DisplayName("a copy left behind by a folder that is gone is ignored, not reported forever")
    void registrationsOfDeletedFoldersAreIgnored() throws IOException {
        fakeCli();
        writeCommandFile(packagedCommand());
        userScopeUrl = ENDPOINT;
        folderScoped.add(Path.of("/Users/someone/Projects/deleted-last-year"));
        writeConfiguration();

        ClaudeCodeInstaller.Installation status = installer().status();

        assertThat(status.folderScoped()).isEmpty();
        assertThat(status.status()).isEqualTo(ClaudeCodeInstaller.CONNECTED);
    }

    @Test
    @DisplayName("installing removes the old registration, adds it at user scope, writes the command")
    void install() throws IOException {
        Path cli = fakeCli();
        registerUserScope("http://localhost:8080/mcp");
        runner = realistic;

        ClaudeCodeInstaller.Installation status = installer().install();

        assertThat(commands).containsExactly(
                List.of(cli.toString(), "mcp", "remove", "--scope", "user", "rekall"),
                List.of(cli.toString(), "mcp", "add", "--scope", "user", "--transport", "http", "rekall", ENDPOINT));
        assertThat(Files.readString(home.resolve(".claude/commands/rk.md"))).isEqualTo(packagedCommand());
        assertThat(status.status()).isEqualTo(ClaudeCodeInstaller.CONNECTED);
        assertThat(status.registeredUrl()).isEqualTo(ENDPOINT);
    }

    @Test
    @DisplayName("installing clears the copy a folder carried, from inside that folder")
    void installClearsFolderScopedCopies() throws IOException {
        Path cli = fakeCli();
        Path carrying = projectFolder("carrying");
        registerFolderScope(carrying);
        runner = realistic;

        ClaudeCodeInstaller.Installation status = installer().install();

        assertThat(commands).contains(List.of(cli.toString(), "mcp", "remove", "--scope", "local", "rekall"));
        // The directory is the whole of what makes that removal address this folder's copy.
        assertThat(directories).endsWith(carrying);
        assertThat(status.folderScoped()).isEmpty();
        assertThat(status.status()).isEqualTo(ClaudeCodeInstaller.CONNECTED);
    }

    @Test
    @DisplayName("a folder that refuses to give up its copy does not fail the install")
    void aFolderThatRefusesIsNotFatal() throws IOException {
        fakeCli();
        registerFolderScope(projectFolder("carrying"));
        runner = (command, environment, directory) -> {
            commands.add(command);
            if (command.contains("local")) {
                return new ClaudeCodeInstaller.Outcome(1, "Error: no such server");
            }
            if (command.contains("add")) {
                registerUserScope(command.getLast());
            }
            return new ClaudeCodeInstaller.Outcome(0, "");
        };

        ClaudeCodeInstaller.Installation status = installer().install();

        assertThat(status.registeredUrl()).isEqualTo(ENDPOINT);
        assertThat(Files.readString(home.resolve(".claude/commands/rk.md"))).isEqualTo(packagedCommand());
    }

    @Test
    @DisplayName("installing over a correct registration is a no-op that still reports CONNECTED")
    void installIsIdempotent() throws IOException {
        fakeCli();
        registerUserScope(ENDPOINT);
        writeCommandFile(packagedCommand());

        assertThat(installer().install().status()).isEqualTo(ClaudeCodeInstaller.CONNECTED);
    }

    @Test
    @DisplayName("without a CLI, installing refuses and hands over the command instead")
    void installWithoutCli() {
        assertThatThrownBy(() -> installer().install())
                .isInstanceOf(ConflictException.class)
                .hasMessageContaining("claude mcp add --scope user --transport http rekall " + ENDPOINT);

        assertThat(commands).isEmpty();
    }

    @Test
    @DisplayName("a CLI that refuses the registration surfaces its own words, and writes nothing")
    void installWhenTheCliFails() throws IOException {
        fakeCli();
        runner = (command, environment, directory) -> {
            commands.add(command);
            return command.contains("add")
                    ? new ClaudeCodeInstaller.Outcome(1, "Error: invalid transport")
                    : new ClaudeCodeInstaller.Outcome(0, "");
        };

        assertThatThrownBy(() -> installer().install())
                .isInstanceOf(ConflictException.class)
                .hasMessageContaining("invalid transport");

        assertThat(Files.exists(home.resolve(".claude/commands/rk.md"))).isFalse();
    }

    // ------------------------------------------------------------------ helpers

    private ClaudeCodeInstaller installer() {
        return new ClaudeCodeInstaller(
                home, ENDPOINT, (command, environment, directory) -> runner.run(command, environment, directory), "");
    }

    /** An executable file where Claude Code installs itself, so the lookup finds it without a PATH. */
    private Path fakeCli() throws IOException {
        Path cli = home.resolve(".local/bin/claude");
        Files.createDirectories(cli.getParent());
        Files.writeString(cli, "#!/bin/sh\n");
        assertThat(cli.toFile().setExecutable(true)).isTrue();
        return cli;
    }

    /** A folder that exists, because a registration in one that does not is not reported. */
    private Path projectFolder(String name) throws IOException {
        return Files.createDirectories(home.resolve("Projects").resolve(name));
    }

    private void registerUserScope(String url) {
        userScopeUrl = url;
        writeConfiguration();
    }

    /** The entry a {@code claude mcp add} without a scope leaves under the folder it ran in. */
    private void registerFolderScope(Path folder) {
        folderScoped.add(folder);
        writeConfiguration();
    }

    /** {@code ~/.claude.json} as Claude Code writes it: one file holding both scopes. */
    private void writeConfiguration() {
        String user = userScopeUrl == null
                ? ""
                : "\"mcpServers\": { \"rekall\": { \"type\": \"http\", \"url\": \"%s\" } },\n  "
                        .formatted(userScopeUrl);
        String projects = folderScoped.stream()
                .map(folder -> "\"%s\": { \"mcpServers\": { \"rekall\": { \"url\": \"%s\" } } }"
                        .formatted(folder, ENDPOINT))
                .collect(Collectors.joining(",\n    "));
        try {
            Files.writeString(home.resolve(".claude.json"),
                    "{\n  %s\"projects\": {\n    %s\n  }\n}".formatted(user, projects));
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    private void writeCommandFile(String content) throws IOException {
        Path file = home.resolve(".claude/commands/rk.md");
        Files.createDirectories(file.getParent());
        Files.writeString(file, content);
    }

    private String packagedCommand() throws IOException {
        try (var stream = ClaudeCodeInstaller.class.getResourceAsStream("/claude/commands/rk.md")) {
            assertThat(stream).as("the /rk command shipped with this build").isNotNull();
            return new String(stream.readAllBytes(), java.nio.charset.StandardCharsets.UTF_8);
        }
    }
}
