package dev.rekall.claude;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.TimeUnit;

/**
 * Reads the OAuth access token Claude Code keeps for the logged-in account: the macOS keychain
 * ({@code Claude Code-credentials}) first, then {@code ~/.claude/.credentials.json}. Never writes or
 * refreshes; a missing token is reported as absent.
 */
@Component
@Slf4j
public class ClaudeCredentials {

    private static final String KEYCHAIN_SERVICE = "Claude Code-credentials";
    private static final long KEYCHAIN_TIMEOUT_SECONDS = 5;

    private final ObjectMapper mapper = new ObjectMapper();
    private final Path home;
    private final boolean macos;

    public ClaudeCredentials() {
        this(
                Path.of(System.getProperty("user.home", "")),
                System.getProperty("os.name", "").toLowerCase().contains("mac"));
    }

    ClaudeCredentials(Path home, boolean macos) {
        this.home = home;
        this.macos = macos;
    }

    public Optional<String> accessToken() {
        Optional<String> fromKeychain = macos ? readKeychain() : Optional.empty();
        return fromKeychain.isPresent() ? fromKeychain : readCredentialsFile();
    }

    private Optional<String> readKeychain() {
        try {
            Process process = new ProcessBuilder(
                    "security", "find-generic-password", "-s", KEYCHAIN_SERVICE, "-w")
                    .redirectErrorStream(false)
                    .start();
            String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
            boolean finished = process.waitFor(KEYCHAIN_TIMEOUT_SECONDS, TimeUnit.SECONDS);
            if (!finished) {
                process.destroyForcibly();
                log.debug("Timed out reading the Claude Code keychain entry");
                return Optional.empty();
            }
            if (process.exitValue() != 0) {
                return Optional.empty();
            }
            return tokenFrom(output.strip());
        } catch (IOException | InterruptedException failure) {
            if (failure instanceof InterruptedException) {
                Thread.currentThread().interrupt();
            }
            log.debug("Could not read the Claude Code keychain entry: {}", failure.getMessage());
            return Optional.empty();
        }
    }

    private Optional<String> readCredentialsFile() {
        Path path = home.resolve(".claude/.credentials.json");
        if (!Files.isReadable(path)) {
            return Optional.empty();
        }
        try {
            return tokenFrom(Files.readString(path, StandardCharsets.UTF_8));
        } catch (IOException failure) {
            log.debug("Could not read {}: {}", path, failure.getMessage());
            return Optional.empty();
        }
    }

    private Optional<String> tokenFrom(String json) {
        if (json == null || json.isBlank()) {
            return Optional.empty();
        }
        try {
            JsonNode token = mapper.readTree(json).path("claudeAiOauth").path("accessToken");
            String value = token.asText("").strip();
            return value.isEmpty() ? Optional.empty() : Optional.of(value);
        } catch (IOException parseFailure) {
            log.debug("Claude Code credentials were not the expected JSON: {}", parseFailure.getMessage());
            return Optional.empty();
        }
    }

    List<String> sources() {
        return macos
                ? List.of("keychain:" + KEYCHAIN_SERVICE, home.resolve(".claude/.credentials.json").toString())
                : List.of(home.resolve(".claude/.credentials.json").toString());
    }
}
