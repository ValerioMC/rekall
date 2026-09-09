package dev.rekall.api.stream;

import dev.rekall.domain.step.StepStreamEvent;
import dev.rekall.domain.wrapup.WrapupStreamEvent;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.util.List;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatCode;

/**
 * The fan-out list, and the one thing that matters for shutdown: when the context closes, every
 * held connection is released so {@code server.shutdown: graceful} has no async request to wait
 * on.
 */
class StepEventStreamTest {

    @Test
    @DisplayName("each open() connection is counted, then dropped when the context closes")
    void releasesEveryClientOnShutdown() {
        StepEventStream stream = new StepEventStream();
        stream.open();
        stream.open();
        stream.open();
        assertThat(stream.clientCount()).isEqualTo(3);

        stream.releaseOnShutdown();

        assertThat(stream.clientCount())
                .as("graceful shutdown must find nothing holding a request open")
                .isZero();
    }

    @Test
    @DisplayName("a step change after shutdown reaches nobody and does not throw")
    void eventAfterShutdownIsANoOp() {
        StepEventStream stream = new StepEventStream();
        stream.open();
        stream.releaseOnShutdown();

        assertThatCode(() -> stream.onStepChange(new StepStreamEvent(UUID.randomUUID(), List.of())))
                .doesNotThrowAnyException();
        assertThat(stream.clientCount()).isZero();
    }

    @Test
    @DisplayName("a wrapup change after shutdown reaches nobody and does not throw")
    void wrapupEventAfterShutdownIsANoOp() {
        StepEventStream stream = new StepEventStream();
        stream.open();
        stream.releaseOnShutdown();

        assertThatCode(() -> stream.onWrapupChange(WrapupStreamEvent.deleted(UUID.randomUUID())))
                .doesNotThrowAnyException();
        assertThat(stream.clientCount()).isZero();
    }

    @Test
    @DisplayName("shutdown with no console connected is harmless")
    void shutdownWithNoClients() {
        StepEventStream stream = new StepEventStream();

        assertThatCode(stream::releaseOnShutdown).doesNotThrowAnyException();
        assertThat(stream.clientCount()).isZero();
    }

    @Test
    @DisplayName("an emitter already completed from the client side does not break the sweep")
    void toleratesAnAlreadyCompletedEmitter() {
        StepEventStream stream = new StepEventStream();
        stream.open();

        assertThatCode(stream::releaseOnShutdown).doesNotThrowAnyException();

        // A second sweep, as if the event fired twice, still finds a clean list.
        assertThatCode(stream::releaseOnShutdown).doesNotThrowAnyException();
        assertThat(stream.clientCount()).isZero();
    }
}
