package dev.rekall.claude;

import com.sun.net.httpserver.HttpServer;
import dev.rekall.claude.ClaudeUsageView.Severity;
import dev.rekall.claude.ClaudeUsageView.Status;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.io.OutputStream;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.Optional;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * The shaping of Anthropic's OAuth usage response into the console meter's rows, and the three
 * ways the figures can be absent. A throwaway {@link HttpServer} stands in for the endpoint.
 */
class ClaudeUsageServiceTest {

    private static final String SAMPLE = """
            {"five_hour":{"utilization":80.0,"resets_at":"2026-09-08T22:20:00.100547+00:00"},
             "seven_day":{"utilization":50.0,"resets_at":"2026-09-10T14:00:00.100568+00:00"},
             "seven_day_opus":{"utilization":97.5,"resets_at":"2026-09-10T14:00:00+00:00"},
             "seven_day_sonnet":null}
            """;

    private HttpServer server;

    @AfterEach
    void stop() {
        if (server != null) {
            server.stop(0);
        }
    }

    private ClaudeUsageService serviceFor(String token, int status, String body, AtomicInteger hits)
            throws IOException {
        server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
        server.createContext("/usage", exchange -> {
            if (hits != null) {
                hits.incrementAndGet();
            }
            byte[] payload = body.getBytes(StandardCharsets.UTF_8);
            exchange.sendResponseHeaders(status, payload.length == 0 ? -1 : payload.length);
            try (OutputStream out = exchange.getResponseBody()) {
                out.write(payload);
            }
        });
        server.start();
        ClaudeCredentials credentials = mock(ClaudeCredentials.class);
        when(credentials.accessToken()).thenReturn(Optional.ofNullable(token));
        String url = "http://127.0.0.1:" + server.getAddress().getPort() + "/usage";
        return new ClaudeUsageService(credentials, url);
    }

    @Test
    @DisplayName("a 200 turns each present window into a row, skips the null ones, grades severity by percent")
    void mapsWindows() throws IOException {
        ClaudeUsageView view = serviceFor("sk-token", 200, SAMPLE, null).current();

        assertThat(view.status()).isEqualTo(Status.OK);
        assertThat(view.limits()).extracting(ClaudeUsageView.Limit::key)
                .containsExactly("session", "weekly_all", "weekly_opus");

        ClaudeUsageView.Limit session = view.limits().get(0);
        assertThat(session.percent()).isEqualTo(80.0);
        assertThat(session.severity()).isEqualTo(Severity.WARNING);
        assertThat(session.resetsAt()).isNotNull();

        assertThat(view.limits().get(1).severity()).isEqualTo(Severity.NORMAL);
        assertThat(view.limits().get(2).severity()).isEqualTo(Severity.CRITICAL);
    }

    @Test
    @DisplayName("no token from Claude Code is reported as unauthenticated, not as an empty meter")
    void noToken() throws IOException {
        ClaudeUsageView view = serviceFor(null, 200, SAMPLE, null).current();

        assertThat(view.status()).isEqualTo(Status.UNAUTHENTICATED);
        assertThat(view.limits()).isEmpty();
    }

    @Test
    @DisplayName("a 401 from Anthropic is unauthenticated")
    void rejectedToken() throws IOException {
        ClaudeUsageView view = serviceFor("stale", 401, "", null).current();

        assertThat(view.status()).isEqualTo(Status.UNAUTHENTICATED);
    }

    @Test
    @DisplayName("a 500 with no earlier good read is unavailable")
    void endpointDown() throws IOException {
        ClaudeUsageView view = serviceFor("sk-token", 500, "", null).current();

        assertThat(view.status()).isEqualTo(Status.UNAVAILABLE);
    }

    @Test
    @DisplayName("a second read inside the cache window does not hit the endpoint again")
    void cached() throws IOException {
        AtomicInteger hits = new AtomicInteger();
        ClaudeUsageService service = serviceFor("sk-token", 200, SAMPLE, hits);

        service.current();
        service.current();

        assertThat(hits).hasValue(1);
    }

    @Test
    @DisplayName("a poll in flight when the context closes is cut loose, not left to time out")
    void releasedOnShutdown() throws Exception {
        CountDownLatch reached = new CountDownLatch(1);
        CountDownLatch release = new CountDownLatch(1);
        server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
        server.createContext("/usage", exchange -> {
            reached.countDown();
            try {
                release.await(10, TimeUnit.SECONDS);
            } catch (InterruptedException interrupted) {
                Thread.currentThread().interrupt();
            }
            exchange.sendResponseHeaders(200, -1);
            exchange.close();
        });
        server.start();
        ClaudeCredentials credentials = mock(ClaudeCredentials.class);
        when(credentials.accessToken()).thenReturn(Optional.of("sk-token"));
        String url = "http://127.0.0.1:" + server.getAddress().getPort() + "/usage";
        ClaudeUsageService service = new ClaudeUsageService(credentials, url);

        AtomicReference<ClaudeUsageView> result = new AtomicReference<>();
        Thread poll = new Thread(() -> result.set(service.current()), "usage-poll");
        poll.start();
        try {
            assertThat(reached.await(5, TimeUnit.SECONDS)).isTrue();

            service.releaseOnShutdown();

            assertThat(poll.join(Duration.ofSeconds(3))).isTrue();
            assertThat(result.get().status()).isEqualTo(Status.UNAVAILABLE);
        } finally {
            release.countDown();
            poll.join();
        }
    }

    @Test
    @DisplayName("after the context closes no further call reaches the endpoint")
    void quietAfterShutdown() throws IOException {
        AtomicInteger hits = new AtomicInteger();
        ClaudeUsageService service = serviceFor("sk-token", 200, SAMPLE, hits);

        service.current();
        service.releaseOnShutdown();
        ClaudeUsageView after = service.current();

        assertThat(hits).hasValue(1);
        assertThat(after.status()).isEqualTo(Status.OK);
    }
}
