package dev.rekall.claude;

import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.context.event.ApplicationReadyEvent;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Component;

import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;

/**
 * The environment a new window of the user's own terminal would have. An app opened from Finder
 * inherits launchd's bare {@code PATH}, so anything a shell profile adds ({@code ~/.cargo/bin},
 * nvm, pyenv, sdkman) is missing from it. This asks the login shell itself: {@code $SHELL -i -l -c}
 * prints its environment between two markers, which keeps whatever the profile echoes out of it.
 *
 * <p>Resolved in the background once the app is up and cached. Every read hands back the cached
 * copy and starts a refresh, so a tool installed while Rekall runs reaches the next terminal but
 * one without a restart. A shell that fails, prints no markers or outlives the timeout leaves the
 * last good copy in place, or this process's own environment when there is none. Windows has no
 * login shell to ask and gets the process environment.
 */
@Component
@Slf4j
public class LoginShellEnvironment {

    /** Set while resolving, so a profile can skip slow work: {@code [[ -n $REKALL_RESOLVING_ENVIRONMENT ]] && return}. */
    static final String RESOLVING_FLAG = "REKALL_RESOLVING_ENVIRONMENT";

    /** Variables that describe the resolving shell itself, not the user's setup. */
    private static final Set<String> SHELL_SESSION_VARIABLES = Set.of("_", "SHLVL", "PWD", "OLDPWD", RESOLVING_FLAG);

    private final Optional<Path> shell;
    private final long timeoutSeconds;

    private final AtomicReference<Map<String, String>> latest = new AtomicReference<>();
    private final AtomicReference<CompletableFuture<Map<String, String>>> inflight = new AtomicReference<>();

    public LoginShellEnvironment(
            @Value("${rekall.terminal.shell:}") String shellOverride,
            @Value("${rekall.terminal.shell-environment-timeout-seconds:10}") long timeoutSeconds) {
        this.shell = chooseShell(shellOverride);
        this.timeoutSeconds = timeoutSeconds;
    }

    @EventListener(ApplicationReadyEvent.class)
    void warm() {
        refresh();
    }

    /** The cached login-shell environment, waiting for the first resolution if none has finished yet. */
    public Map<String, String> current() {
        Map<String, String> known = latest.get();
        CompletableFuture<Map<String, String>> pending = refresh();
        return known != null ? known : pending.join();
    }

    /** Start a resolution unless one is already running, and return the one that is. */
    private CompletableFuture<Map<String, String>> refresh() {
        CompletableFuture<Map<String, String>> fresh = new CompletableFuture<>();
        CompletableFuture<Map<String, String>> running = inflight.compareAndExchange(null, fresh);
        if (running != null) {
            return running;
        }
        Thread.ofVirtual().name("login-shell-environment").start(() -> {
            try {
                resolve().ifPresent(latest::set);
            } finally {
                Map<String, String> known = latest.get();
                inflight.set(null);
                fresh.complete(known != null ? known : System.getenv());
            }
        });
        return fresh;
    }

    /** One run of the login shell. Empty on any failure, which is logged with its reason. */
    Optional<Map<String, String>> resolve() {
        if (shell.isEmpty()) {
            return Optional.empty();
        }
        String marker = "rekall-" + UUID.randomUUID();
        String script = "printf '%s' '" + marker + "'; /usr/bin/env -0; printf '%s' '" + marker + "'";
        Path output = null;
        Process process = null;
        try {
            output = Files.createTempFile("rekall-shell-environment", ".out");
            ProcessBuilder builder = new ProcessBuilder(List.of(shell.get().toString(), "-i", "-l", "-c", script))
                    .redirectInput(ProcessBuilder.Redirect.from(new File("/dev/null")))
                    .redirectOutput(output.toFile())
                    .redirectError(ProcessBuilder.Redirect.DISCARD);
            builder.environment().put(RESOLVING_FLAG, "1");
            process = builder.start();
            if (!process.waitFor(timeoutSeconds, TimeUnit.SECONDS)) {
                log.warn("{} took longer than {}s to print its environment; terminals keep the last one known",
                        shell.get(), timeoutSeconds);
                return Optional.empty();
            }
            Optional<Map<String, String>> environment =
                    parse(Files.readString(output, StandardCharsets.UTF_8), marker);
            if (environment.isEmpty()) {
                log.warn("{} exited with code {} without printing its environment", shell.get(), process.exitValue());
            }
            return environment;
        } catch (IOException failed) {
            log.warn("Could not read the environment of {}: {}", shell.get(), failed.getMessage());
            return Optional.empty();
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            return Optional.empty();
        } finally {
            if (process != null && process.isAlive()) {
                process.descendants().forEach(ProcessHandle::destroyForcibly);
                process.destroyForcibly();
            }
            deleteQuietly(output);
        }
    }

    /** The {@code env -0} block between the two markers, minus the resolving shell's own variables. */
    static Optional<Map<String, String>> parse(String output, String marker) {
        int start = output.indexOf(marker);
        int end = output.lastIndexOf(marker);
        if (start < 0 || end <= start) {
            return Optional.empty();
        }
        Map<String, String> environment = new HashMap<>();
        for (String entry : output.substring(start + marker.length(), end).split("\0")) {
            int equals = entry.indexOf('=');
            if (equals <= 0) {
                continue;
            }
            String key = entry.substring(0, equals);
            if (!SHELL_SESSION_VARIABLES.contains(key)) {
                environment.put(key, entry.substring(equals + 1));
            }
        }
        return environment.isEmpty() ? Optional.empty() : Optional.of(Map.copyOf(environment));
    }

    private static Optional<Path> chooseShell(String override) {
        if (System.getProperty("os.name", "").toLowerCase(Locale.ROOT).startsWith("windows")) {
            return Optional.empty();
        }
        String configured = override == null || override.isBlank() ? System.getenv("SHELL") : override.strip();
        if (configured != null && !configured.isBlank()) {
            return Optional.of(Path.of(configured));
        }
        Path zsh = Path.of("/bin/zsh");
        return Optional.of(Files.isExecutable(zsh) ? zsh : Path.of("/bin/sh"));
    }

    private static void deleteQuietly(Path file) {
        if (file == null) {
            return;
        }
        try {
            Files.deleteIfExists(file);
        } catch (IOException failed) {
            log.debug("Could not delete {}: {}", file, failed.getMessage());
        }
    }
}
