package dev.rekall.claude;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Clock;
import java.time.Instant;
import java.time.ZoneOffset;
import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

/** Which stored token is handed out: the keychain items and the credentials file, by freshness. */
class ClaudeCredentialsTest {

    private static final Instant NOW = Instant.parse("2026-09-15T12:00:00Z");

    @TempDir
    Path home;

    private ClaudeCredentials credentials(String... keychainSecrets) {
        return new ClaudeCredentials(home, () -> List.of(keychainSecrets), Clock.fixed(NOW, ZoneOffset.UTC));
    }

    @Test
    @DisplayName("reads the access token out of the credentials file")
    void readsFromFile() throws IOException {
        writeCredentials("{\"claudeAiOauth\":{\"accessToken\":\"sk-ant-oat01-abc\"}}");

        assertThat(credentials().accessToken()).contains("sk-ant-oat01-abc");
    }

    @Test
    @DisplayName("no file and no keychain item means no token, not an error")
    void missingFile() {
        assertThat(credentials().accessToken()).isEmpty();
    }

    @Test
    @DisplayName("a file that is not the expected shape yields no token")
    void wrongShape() throws IOException {
        writeCredentials("{\"something\":\"else\"}");

        assertThat(credentials().accessToken()).isEmpty();
    }

    @Test
    @DisplayName("a blank token is treated as absent")
    void blankToken() throws IOException {
        writeCredentials("{\"claudeAiOauth\":{\"accessToken\":\"  \"}}");

        assertThat(credentials().accessToken()).isEmpty();
    }

    @Test
    @DisplayName("of several keychain items the one Claude Code refreshed last wins, whatever its order")
    void freshestKeychainItemWins() {
        String stale = secret("stale", NOW.minusSeconds(3600));
        String live = secret("live", NOW.plusSeconds(3600));

        assertThat(credentials(stale, live).accessToken()).contains("live");
        assertThat(credentials(live, stale).accessToken()).contains("live");
    }

    @Test
    @DisplayName("a keychain item beats the credentials file when it expires later")
    void keychainBeatsFileByExpiry() throws IOException {
        writeCredentials(secret("from-file", NOW.plusSeconds(60)));

        assertThat(credentials(secret("from-keychain", NOW.plusSeconds(120))).accessToken())
                .contains("from-keychain");
    }

    @Test
    @DisplayName("a token past its expiry is absent: it would only be refused, and refusals get this machine rate limited")
    void expiredTokenIsAbsent() {
        assertThat(credentials(secret("expired", NOW.minusSeconds(1))).accessToken()).isEmpty();
    }

    @Test
    @DisplayName("a token with no expiry is kept, but loses to one that says when it expires")
    void unknownExpiryRanksLowest() {
        assertThat(credentials("{\"claudeAiOauth\":{\"accessToken\":\"undated\"}}").accessToken())
                .contains("undated");
        assertThat(credentials(
                "{\"claudeAiOauth\":{\"accessToken\":\"undated\"}}", secret("dated", NOW.plusSeconds(10)))
                .accessToken())
                .contains("dated");
    }

    @Test
    @DisplayName("an unreadable keychain item is skipped, not fatal")
    void malformedKeychainItemSkipped() {
        assertThat(credentials("not json", secret("good", NOW.plusSeconds(10))).accessToken())
                .contains("good");
    }

    private static String secret(String token, Instant expiresAt) {
        return "{\"claudeAiOauth\":{\"accessToken\":\"" + token + "\",\"expiresAt\":" + expiresAt.toEpochMilli() + "}}";
    }

    private void writeCredentials(String json) throws IOException {
        Path dir = Files.createDirectories(home.resolve(".claude"));
        Files.writeString(dir.resolve(".credentials.json"), json);
    }
}
