package dev.rekall.claude;

import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * Finds the {@code claude} binary and the environment to run it with. A desktop app starts with a
 * stripped {@code PATH}, so the Claude Code install locations are tried first;
 * {@code rekall.claude.cli-path} overrides everything.
 */
@Component
@Slf4j
public class ClaudeCli {

    private static final List<String> HOME_RELATIVE =
            List.of(".local/bin/claude", ".claude/local/claude", "bin/claude");
    private static final List<String> KNOWN_DIRECTORIES =
            List.of("/opt/homebrew/bin", "/usr/local/bin", "/usr/bin");

    private final String override;
    private final Path home;

    public ClaudeCli(@Value("${rekall.claude.cli-path:}") String override) {
        this.override = override == null ? "" : override.strip();
        this.home = Path.of(System.getProperty("user.home", ""));
    }

    public Optional<Path> locate() {
        if (!override.isBlank()) {
            Path path = Path.of(override);
            return Files.isExecutable(path) ? Optional.of(path) : Optional.empty();
        }
        for (String candidate : HOME_RELATIVE) {
            Path path = home.resolve(candidate);
            if (Files.isExecutable(path)) {
                return Optional.of(path);
            }
        }
        for (String directory : searchDirectories()) {
            Path path = Path.of(directory).resolve("claude");
            if (Files.isExecutable(path)) {
                return Optional.of(path);
            }
        }
        return Optional.empty();
    }

    /** {@code HOME} and a {@code PATH} that still finds the CLI's own directories. */
    public Map<String, String> environment() {
        Map<String, String> environment = new HashMap<>();
        if (!home.toString().isBlank()) {
            environment.put("HOME", home.toString());
        }
        String current = System.getenv("PATH");
        String known = String.join(":", KNOWN_DIRECTORIES);
        environment.put("PATH", current == null || current.isBlank() ? known : current + ":" + known);
        return environment;
    }

    private List<String> searchDirectories() {
        String path = System.getenv("PATH");
        if (path == null || path.isBlank()) {
            return KNOWN_DIRECTORIES;
        }
        return java.util.stream.Stream.concat(
                        java.util.Arrays.stream(path.split(":")).filter(part -> !part.isBlank()),
                        KNOWN_DIRECTORIES.stream())
                .toList();
    }
}
