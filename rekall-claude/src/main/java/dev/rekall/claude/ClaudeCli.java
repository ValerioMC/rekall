package dev.rekall.claude;

import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Stream;

/**
 * Finds the {@code claude} binary and the environment to run it with. The environment is the user's
 * login shell's ({@link LoginShellEnvironment}), so a terminal opened here finds what their own
 * terminal finds; the Claude Code install locations are still tried first, and
 * {@code rekall.claude.cli-path} overrides everything.
 */
@Component
@Slf4j
public class ClaudeCli {

    private static final List<String> HOME_RELATIVE =
            List.of(".local/bin/claude", ".claude/local/claude", "bin/claude");
    private static final List<String> KNOWN_DIRECTORIES =
            List.of("/opt/homebrew/bin", "/usr/local/bin", "/usr/bin");

    /** Set by a running Claude Code session; inherited, they make the new {@code claude} refuse to nest. */
    private static final Set<String> SESSION_MARKERS = Set.of("CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT");

    private final String override;
    private final Path home;
    private final LoginShellEnvironment loginShell;

    public ClaudeCli(@Value("${rekall.claude.cli-path:}") String override, LoginShellEnvironment loginShell) {
        this.override = override == null ? "" : override.strip();
        this.home = Path.of(System.getProperty("user.home", ""));
        this.loginShell = loginShell;
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
        for (String directory : searchPath(environment().get("PATH"))) {
            Path path = Path.of(directory).resolve("claude");
            if (Files.isExecutable(path)) {
                return Optional.of(path);
            }
        }
        return Optional.empty();
    }

    /**
     * The login shell's environment with {@code HOME} set, a {@code PATH} that still reaches the
     * CLI's usual directories, and no marker of an enclosing Claude Code session.
     */
    public Map<String, String> environment() {
        Map<String, String> environment = new HashMap<>(loginShell.current());
        SESSION_MARKERS.forEach(environment::remove);
        if (!home.toString().isBlank()) {
            environment.putIfAbsent("HOME", home.toString());
        }
        environment.put("PATH", String.join(":", searchPath(environment.get("PATH"))));
        return environment;
    }

    /** The directories of {@code path} in order, then any of the usual install directories it lacks. */
    private static List<String> searchPath(String path) {
        Stream<String> declared = path == null ? Stream.empty() : Arrays.stream(path.split(":"));
        return Stream.concat(declared.filter(part -> !part.isBlank()), KNOWN_DIRECTORIES.stream())
                .distinct()
                .toList();
    }
}
