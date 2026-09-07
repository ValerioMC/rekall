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

@Slf4j
public class ClaudeCodeInstaller {

    public static final String SERVER_NAME = "rekall";

    public static final String CONNECTED = "CONNECTED";
    public static final String OUTDATED = "OUTDATED";
    public static final String NOT_CONNECTED = "NOT_CONNECTED";
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

    public record Installation(
            String status,
            String endpoint,
            String registeredUrl,
            List<String> folderScoped,
            boolean commandInstalled,
            String cliPath,
            String manualCommand) {
    }

    @FunctionalInterface
    public interface CommandRunner {

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

    private String registeredUrl() {
        return configuration().at("/mcpServers/" + SERVER_NAME + "/url").textValue();
    }

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

    private Path commandFile() {
        return home.resolve(".claude").resolve("commands").resolve(COMMAND_FILE);
    }

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

    private static String defaultSearchPath() {
        String path = System.getenv("PATH");
        return path == null || path.isBlank() ? KNOWN_DIRECTORIES : path + ":" + KNOWN_DIRECTORIES;
    }

    private String manualCommand() {
        return "claude mcp add --scope user --transport http " + SERVER_NAME + " " + endpoint;
    }

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
