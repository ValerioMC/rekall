package dev.rekall.claude;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Clock;
import java.time.Instant;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.TimeUnit;

/**
 * Reads the OAuth access token Claude Code keeps for the logged-in account: the macOS keychain
 * ({@code Claude Code-credentials}) first, then {@code ~/.claude/.credentials.json}. Never writes or
 * refreshes; a missing token is reported as absent.
 *
 * <p>The keychain can hold more than one item under that service (one per account name a Claude
 * Code release has used), and only one of them is the one Claude Code still refreshes. Every item
 * is read and the token with the latest {@code expiresAt} wins. A token whose {@code expiresAt}
 * has passed is reported as absent rather than sent: Anthropic would only answer 401, and a
 * stream of those is what gets this machine rate limited.
 */
@Component
@Slf4j
public class ClaudeCredentials {

    static final String KEYCHAIN_SERVICE = "Claude Code-credentials";

    /** Where the keychain half of the read comes from; the real one shells out to {@code security}. */
    interface KeychainSource {
        List<String> secrets();
    }

    /** One stored token and when Claude Code said it stops working; {@code expiresAt} may be null. */
    record StoredToken(String accessToken, Instant expiresAt) {
        boolean expiredAt(Instant now) {
            return expiresAt != null && !expiresAt.isAfter(now);
        }
    }

    private final ObjectMapper mapper = new ObjectMapper();
    private final Path home;
    private final KeychainSource keychain;
    private final Clock clock;

    public ClaudeCredentials() {
        this(
                Path.of(System.getProperty("user.home", "")),
                System.getProperty("os.name", "").toLowerCase().contains("mac")
                        ? new SecurityCommandKeychain()
                        : List::of,
                Clock.systemUTC());
    }

    ClaudeCredentials(Path home, KeychainSource keychain, Clock clock) {
        this.home = home;
        this.keychain = keychain;
        this.clock = clock;
    }

    /** The freshest token that is still valid, or empty when every stored one is missing or expired. */
    public Optional<String> accessToken() {
        List<StoredToken> stored = new ArrayList<>();
        for (String secret : keychain.secrets()) {
            tokenFrom(secret).ifPresent(stored::add);
        }
        readCredentialsFile().ifPresent(stored::add);
        Optional<StoredToken> freshest = stored.stream()
                .max(Comparator.comparing(StoredToken::expiresAt, Comparator.nullsFirst(Comparator.naturalOrder())));
        if (freshest.isEmpty()) {
            return Optional.empty();
        }
        if (freshest.get().expiredAt(clock.instant())) {
            log.debug("Every stored Claude Code token has expired, the latest at {}", freshest.get().expiresAt());
            return Optional.empty();
        }
        return Optional.of(freshest.get().accessToken());
    }

    private Optional<StoredToken> readCredentialsFile() {
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

    private Optional<StoredToken> tokenFrom(String json) {
        if (json == null || json.isBlank()) {
            return Optional.empty();
        }
        try {
            JsonNode oauth = mapper.readTree(json).path("claudeAiOauth");
            String value = oauth.path("accessToken").asText("").strip();
            if (value.isEmpty()) {
                return Optional.empty();
            }
            JsonNode expiresAt = oauth.path("expiresAt");
            Instant expiry = expiresAt.isNumber() ? Instant.ofEpochMilli(expiresAt.asLong()) : null;
            return Optional.of(new StoredToken(value, expiry));
        } catch (IOException parseFailure) {
            log.debug("Claude Code credentials were not the expected JSON: {}", parseFailure.getMessage());
            return Optional.empty();
        }
    }

    /**
     * The macOS keychain through the {@code security} command. {@code find-generic-password}
     * returns one item per call, and picks the first when several share a service, so the
     * account names are listed first with {@code dump-keychain} and each is read on its own.
     */
    @Slf4j
    static final class SecurityCommandKeychain implements KeychainSource {

        private static final long COMMAND_TIMEOUT_SECONDS = 5;

        @Override
        public List<String> secrets() {
            Set<String> accounts = accountsHoldingTheService();
            if (accounts.isEmpty()) {
                return secret(null).map(List::of).orElse(List.of());
            }
            List<String> secrets = new ArrayList<>();
            for (String account : accounts) {
                secret(account).ifPresent(secrets::add);
            }
            return secrets;
        }

        private Set<String> accountsHoldingTheService() {
            Set<String> accounts = new LinkedHashSet<>();
            Optional<String> dump = run("security", "dump-keychain");
            if (dump.isEmpty()) {
                return accounts;
            }
            String account = null;
            for (String line : dump.get().split("\n")) {
                String trimmed = line.strip();
                if (trimmed.startsWith("keychain:")) {
                    account = null;
                } else if (trimmed.startsWith("\"acct\"<blob>=")) {
                    account = attributeValue(trimmed);
                } else if (trimmed.startsWith("\"svce\"<blob>=")
                        && KEYCHAIN_SERVICE.equals(attributeValue(trimmed))
                        && account != null) {
                    accounts.add(account);
                }
            }
            return accounts;
        }

        private static String attributeValue(String line) {
            int start = line.indexOf("=\"");
            int end = line.lastIndexOf('"');
            return start < 0 || end <= start + 1 ? null : line.substring(start + 2, end);
        }

        private Optional<String> secret(String account) {
            List<String> command = new ArrayList<>(List.of(
                    "security", "find-generic-password", "-s", KEYCHAIN_SERVICE));
            if (account != null) {
                command.addAll(List.of("-a", account));
            }
            command.add("-w");
            return run(command.toArray(String[]::new)).map(String::strip).filter(value -> !value.isEmpty());
        }

        private Optional<String> run(String... command) {
            try {
                Process process = new ProcessBuilder(command).redirectErrorStream(false).start();
                String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
                boolean finished = process.waitFor(COMMAND_TIMEOUT_SECONDS, TimeUnit.SECONDS);
                if (!finished) {
                    process.destroyForcibly();
                    log.debug("Timed out running {}", command[1]);
                    return Optional.empty();
                }
                if (process.exitValue() != 0) {
                    return Optional.empty();
                }
                return Optional.of(output);
            } catch (IOException | InterruptedException failure) {
                if (failure instanceof InterruptedException) {
                    Thread.currentThread().interrupt();
                }
                log.debug("Could not run {}: {}", command[1], failure.getMessage());
                return Optional.empty();
            }
        }
    }
}
