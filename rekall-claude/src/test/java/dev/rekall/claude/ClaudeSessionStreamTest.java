package dev.rekall.claude;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatCode;

/**
 * The per-session fan-out, and the shutdown guarantee it shares with {@code StepEventStream}:
 * when the context closes, every held connection is released so graceful shutdown has no async
 * request to wait on.
 */
class ClaudeSessionStreamTest {

    @Test
    @DisplayName("connections are counted per session id")
    void countsPerSession() {
        ClaudeSessionStream stream = new ClaudeSessionStream();
        UUID one = UUID.randomUUID();
        UUID two = UUID.randomUUID();

        stream.open(one);
        stream.open(one);
        stream.open(two);

        assertThat(stream.clientCount(one)).isEqualTo(2);
        assertThat(stream.clientCount(two)).isEqualTo(1);
        assertThat(stream.clientCount()).isEqualTo(3);
    }

    @Test
    @DisplayName("an event for a session nobody is watching is a no-op")
    void emitToNobody() {
        ClaudeSessionStream stream = new ClaudeSessionStream();

        assertThatCode(() -> stream.emit(UUID.randomUUID(), "message", "x")).doesNotThrowAnyException();
    }

    @Test
    @DisplayName("every connection is released when the context closes")
    void releasesOnShutdown() {
        ClaudeSessionStream stream = new ClaudeSessionStream();
        stream.open(UUID.randomUUID());
        stream.open(UUID.randomUUID());
        assertThat(stream.clientCount()).isEqualTo(2);

        stream.releaseOnShutdown();

        assertThat(stream.clientCount()).isZero();
        assertThatCode(stream::releaseOnShutdown).doesNotThrowAnyException();
    }
}
