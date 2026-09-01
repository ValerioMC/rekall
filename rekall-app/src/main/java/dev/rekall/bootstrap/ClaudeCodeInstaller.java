package dev.rekall.bootstrap;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import dev.rekall.api.service.ConflictException;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;

/**
 * Registers this running instance with Claude Code, and reports whether it still is.
 *
 * <p>Two things have to be in place before {@code /rk} works in a session: the MCP server has to
 * be registered at <em>user</em> scope, and the slash command has to exist in
 * {@code ~/.claude/commands}. Registered at the default <em>local</em> scope instead, the server
 * only exists for the one folder the registration was run from, which looks identical from
 * inside that folder and like nothing at all from anywhere else.
 *
 * <p>The registration itself goes through the {@code claude} CLI rather than through
 * {@code ~/.claude.json} directly. That file belongs to Claude Code, which holds it open and
 * rewrites it whole; a second writer would silently lose whichever of the two wrote first. It is
 * only ever <em>read</em> here, to answer whether the registration is there. The slash command is
 * a plain file write: {@code ~/.claude/commands} is a directory Claude Code only reads.
 */
@Slf4j
public class ClaudeCodeInstaller {

    /** The name the server is registered under, and so the prefix of its tools. */
    public static final String SERVER_NAME = "rekall";

    /** Everything is in place and a new session will find it. */
    public static final String CONNECTED = "CONNECTED";
    /** Registered, but pointing somewhere else or carrying an older copy of the command. */
    public static final String OUTDATED = "OUTDATED";
    /** Claude Code is on this machine and knows nothing about Rekall. */
    public static final String NOT_CONNECTED = "NOT_CONNECTED";
    /** No {@code claude} binary to register with, so the button can only hand over the command. */
    public static final String CLI_MISSING = "CLI_MISSING";

    private static final String COMMAND_FILE = "rk.md";
    private static final String PACKAGED_COMMAND = "/claude/commands/" + COMMAND_FILE;
    private static final List<String> HOME_RELATIVE_BINARIES =
            List.of(".local/bin/claude", ".claude/local/claude", "bin/claude");
    private static final String KNOWN_DIRECTORIES = "/opt/homebrew/bin:/usr/local/bin:/usr/bin";
    private static final long COMMAND_TIMEOUT_SECONDS = 20;

    private final Path home;
    private final String endpoint;
    private final CommandRunner runner;
    private final String searchPath;
    private final ObjectMapper mapper = new ObjectMapper();

    /**
     * What a session would find right now.
     *
     * @param status            one of {@link #CONNECTED}, {@link #OUTDATED}, {@link #NOT_CONNECTED},
     *                          {@link #CLI_MISSING}
     * @param endpoint          the URL this instance is actually serving MCP on
     * @param registeredUrl     what Claude Code has at user scope, or null when it has nothing
     * @param folderScoped      folders that still carry a registration of their own, which is
     *                          what a {@code claude mcp add} without a scope leaves behind, and
     *                          which {@link #install()} clears
     * @param commandInstalled  whether {@code ~/.claude/commands/rk.md} matches the packaged one
     * @param cliPath           the {@code claude} binary found, or null
     * @param manualCommand     the shell equivalent of {@link #install()}, for when there is no CLI
     */
    public record Installation(
            String status,
            String endpoint,
            String registeredUrl,
            List<String> folderScoped,
            boolean commandInstalled,
            String cliPath,
            String manualCommand) {
    }

    /** What running one command produced. Kept as an interface so tests never launch a process. */
    @FunctionalInterface
    public interface CommandRunner {

        /**
         * @param directory where the command runs, which is the whole of what makes a local-scope
         *                  registration local: the same {@code mcp remove} run from two folders
         *                  addresses two different entries
         */
        Outcome run(List<String> command, Map<String, String> environment, Path directory);
    }

    public record Outcome(int exitCode, String output) {
        public boolean succeeded() {
            return exitCode == 0;
        }
    }

    public ClaudeCodeInstaller(String endpoint) {
        this(Path.of(System.getProperty("user.home")), endpoint, ClaudeCodeInstaller::runProcess);
    }

    public ClaudeCodeInstaller(Path home, String endpoint, CommandRunner runner) {
        this(home, endpoint, runner, defaultSearchPath());
    }

    /** The search path is a parameter so a test can describe a machine without Claude Code on it. */
    ClaudeCodeInstaller(Path home, String endpoint, CommandRunner runner, String searchPath) {
        this.home = home;
        this.endpoint = endpoint;
        this.runner = runner;
        this.searchPath = searchPath;
    }

    public Installation status() {
        String registered = registeredUrl();
        String cli = locateCli();
        boolean command = commandIsCurrent();
        // A folder-scope copy keeps this short of connected however right the user-scope one is:
        // inside that folder it is the copy a session finds, so one folder would be answering
        // from somewhere else while the badge said everything was in place.
        List<String> folders = folderScoped();
        String status;
        if (endpoint.equals(registered) && command && folders.isEmpty()) {
            status = CONNECTED;
        } else if (cli == null) {
            status = CLI_MISSING;
        } else if (registered == null) {
            status = NOT_CONNECTED;
        } else {
            status = OUTDATED;
        }
        return new Installation(status, endpoint, registered, folders, command, cli, manualCommand());
    }

    /**
     * Removes whatever user-scope registration is there and writes it again from scratch, then
     * overwrites the slash command. Idempotent by construction: the outcome is the same whether it
     * was missing, stale or already correct, which is what makes it safe to offer as one button.
     *
     * <p>The copies a folder carries of its own go too. What this button promises is one
     * registration that answers from every folder, and a folder-scope copy wins over it inside
     * that folder: leaving one behind leaves one folder where Rekall answers on whatever address
     * was current when it was written.
     */
    public Installation install() {
        String cli = locateCli();
        if (cli == null) {
            throw new ConflictException(
                    "Claude Code's command line tool isn't on this machine, so the registration can't be written for you. "
                            + "Run this instead: " + manualCommand());
        }
        Map<String, String> environment = Map.of("HOME", home.toString());

        Outcome removed = runner.run(
                List.of(cli, "mcp", "remove", "--scope", "user", SERVER_NAME), environment, home);
        if (!removed.succeeded()) {
            log.debug("Nothing to remove before registering {}: {}", SERVER_NAME, removed.output());
        }

        Outcome added = runner.run(
                List.of(cli, "mcp", "add", "--scope", "user", "--transport", "http", SERVER_NAME, endpoint),
                environment, home);
        if (!added.succeeded()) {
            throw new ConflictException("Claude Code refused the registration: " + added.output());
        }

        clearFolderScoped(cli, environment);
        installCommand();
        log.info("Registered {} at user scope on {} and installed the /rk command", SERVER_NAME, endpoint);
        return status();
    }

    /**
     * Removes the folder-scope copies, one command per folder. The removal has to run from inside
     * the folder, because that directory is the entire address of a local-scope entry.
     *
     * <p>A folder that refuses is logged and not fatal. The registration this method was called
     * for is already written by the time it runs, and one folder that keeps its own copy is one
     * folder short of the promise rather than a failed install.
     */
    private void clearFolderScoped(String cli, Map<String, String> environment) {
        for (String folder : folderScoped()) {
            Outcome outcome = runner.run(
                    List.of(cli, "mcp", "remove", "--scope", "local", SERVER_NAME),
                    environment,
                    Path.of(folder));
            if (outcome.succeeded()) {
                log.info("Removed the {} registration {} carried of its own", SERVER_NAME, folder);
            } else {
                log.warn("Could not remove the {} registration in {}: {}", SERVER_NAME, folder, outcome.output());
            }
        }
    }

    // ------------------------------------------------------------------ the registration

    /** The URL registered at user scope, or null. Read-only: this file has another owner. */
    private String registeredUrl() {
        return configuration().at("/mcpServers/" + SERVER_NAME + "/url").textValue();
    }

    /**
     * The folders that carry a registration of their own.
     *
     * <p>{@code claude mcp add} defaults to local scope, which records the server under the one
     * directory it was run from, and inside that directory it is the one a session finds.
     *
     * <p>A folder that no longer exists is left out. No session can be started from it, so its
     * entry cannot shadow anything, and there is no directory to run the removal from either.
     */
    private List<String> folderScoped() {
        JsonNode projects = configuration().path("projects");
        List<String> folders = new ArrayList<>();
        for (Iterator<String> names = projects.fieldNames(); names.hasNext(); ) {
            String folder = names.next();
            if (projects.path(folder).at("/mcpServers/" + SERVER_NAME).isObject()
                    && Files.isDirectory(Path.of(folder))) {
                folders.add(folder);
            }
        }
        return List.copyOf(folders);
    }

    /**
     * Claude Code's own configuration, or an empty node when it is absent or unreadable. Never a
     * failure: not having it only means there is nothing registered yet, which is an answer.
     */
    private JsonNode configuration() {
        Path file = home.resolve(".claude.json");
        if (!Files.isRegularFile(file)) {
            return mapper.createObjectNode();
        }
        try {
            return mapper.readTree(file.toFile());
        } catch (IOException e) {
            log.warn("Could not read {}, reporting Rekall as unregistered: {}", file, e.getMessage());
            return mapper.createObjectNode();
        }
    }

    // ------------------------------------------------------------------ the slash command

    private Path commandFile() {
        return home.resolve(".claude").resolve("commands").resolve(COMMAND_FILE);
    }

    /** True only when the installed command is byte for byte the one shipped with this build. */
    private boolean commandIsCurrent() {
        Path file = commandFile();
        if (!Files.isRegularFile(file)) {
            return false;
        }
        try {
            return Files.readString(file).equals(packagedCommand());
        } catch (IOException e) {
            log.warn("Could not read {}: {}", file, e.getMessage());
            return false;
        }
    }

    private void installCommand() {
        Path file = commandFile();
        try {
            Files.createDirectories(file.getParent());
            Files.writeString(file, packagedCommand());
        } catch (IOException e) {
            throw new UncheckedIOException("Could not write " + file, e);
        }
    }

    private String packagedCommand() {
        try (InputStream stream = ClaudeCodeInstaller.class.getResourceAsStream(PACKAGED_COMMAND)) {
            if (stream == null) {
                throw new IllegalStateException(PACKAGED_COMMAND + " is missing from this build");
            }
            return new String(stream.readAllBytes(), StandardCharsets.UTF_8);
        } catch (IOException e) {
            throw new UncheckedIOException("Could not read " + PACKAGED_COMMAND, e);
        }
    }

    // ------------------------------------------------------------------ the binary

    /**
     * The {@code claude} binary, or null.
     *
     * <p>The per-user install locations are tried first, then every directory on
     * {@link #defaultSearchPath()}.
     */
    private String locateCli() {
        for (String candidate : HOME_RELATIVE_BINARIES) {
            Path path = home.resolve(candidate);
            if (Files.isExecutable(path)) {
                return path.toString();
            }
        }
        if (searchPath == null) {
            return null;
        }
        for (String directory : searchPath.split(":")) {
            if (directory.isBlank()) {
                continue;
            }
            Path candidate = Path.of(directory).resolve("claude");
            if (Files.isExecutable(candidate)) {
                return candidate.toString();
            }
        }
        return null;
    }

    /**
     * {@code PATH} widened with the directories Claude Code actually installs into. A bundle
     * launched from the Finder inherits a minimal environment: no Homebrew, none of what a login
     * shell would have added, so a lookup through {@code PATH} alone would report Claude Code as
     * missing on the machine that is running it.
     */
    private static String defaultSearchPath() {
        String path = System.getenv("PATH");
        return path == null || path.isBlank() ? KNOWN_DIRECTORIES : path + ":" + KNOWN_DIRECTORIES;
    }

    private String manualCommand() {
        return "claude mcp add --scope user --transport http " + SERVER_NAME + " " + endpoint;
    }

    // ------------------------------------------------------------------ running it

    /**
     * Output and errors are read as one stream, because the only use for either is a sentence in
     * the panel saying why the registration did not happen.
     */
    private static Outcome runProcess(List<String> command, Map<String, String> environment, Path directory) {
        ProcessBuilder builder = new ProcessBuilder(command).redirectErrorStream(true);
        builder.directory(directory.toFile());
        builder.environment().putAll(environment);
        Process process = null;
        try {
            process = builder.start();
            String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8).trim();
            if (!process.waitFor(COMMAND_TIMEOUT_SECONDS, TimeUnit.SECONDS)) {
                process.destroyForcibly();
                return new Outcome(-1, "`" + String.join(" ", command) + "` did not finish in "
                        + COMMAND_TIMEOUT_SECONDS + " seconds");
            }
            return new Outcome(process.exitValue(), output);
        } catch (IOException e) {
            return new Outcome(-1, e.getMessage());
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            if (process != null) {
                process.destroyForcibly();
            }
            return new Outcome(-1, "Interrupted while running " + String.join(" ", command));
        }
    }
}
