package dev.rekall.claude;

import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.ContextClosedEvent;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Component;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.io.IOException;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CopyOnWriteArrayList;

/**
 * The live feed for a hosted session, one subscriber list per session id.
 *
 * <p>A console watches exactly the session whose pane is open. Events carry the session's new
 * messages ({@code message}), its status changes ({@code status}) and the moment it ends
 * ({@code ended}); the pane fetches the transcript once over REST and then follows this.
 *
 * <p>Like {@code StepEventStream}, every held connection is released on {@link ContextClosedEvent}
 * so {@code server.shutdown: graceful} has no async request to wait on.
 */
@Component
@Slf4j
public class ClaudeSessionStream {

    private static final long NO_TIMEOUT = 0L;

    private final Map<UUID, List<SseEmitter>> bySession = new ConcurrentHashMap<>();

    public SseEmitter open(UUID sessionId) {
        SseEmitter emitter = new SseEmitter(NO_TIMEOUT);
        List<SseEmitter> clients = bySession.computeIfAbsent(sessionId, key -> new CopyOnWriteArrayList<>());
        emitter.onCompletion(() -> drop(sessionId, emitter));
        emitter.onTimeout(() -> drop(sessionId, emitter));
        emitter.onError(error -> drop(sessionId, emitter));
        clients.add(emitter);
        try {
            emitter.send(SseEmitter.event().name("open").data("ready"));
        } catch (IOException opening) {
            drop(sessionId, emitter);
        }
        return emitter;
    }

    public void emit(UUID sessionId, String name, Object payload) {
        List<SseEmitter> clients = bySession.get(sessionId);
        if (clients == null) {
            return;
        }
        for (SseEmitter emitter : clients) {
            try {
                emitter.send(SseEmitter.event().name(name).data(payload));
            } catch (IOException | IllegalStateException gone) {
                drop(sessionId, emitter);
            }
        }
    }

    @EventListener(ContextClosedEvent.class)
    public void releaseOnShutdown() {
        for (List<SseEmitter> clients : bySession.values()) {
            for (SseEmitter emitter : clients) {
                try {
                    emitter.complete();
                } catch (RuntimeException alreadyClosed) {
                    // The onCompletion/onError callback has it.
                }
            }
        }
        bySession.clear();
    }

    public int clientCount() {
        return bySession.values().stream().mapToInt(List::size).sum();
    }

    public int clientCount(UUID sessionId) {
        List<SseEmitter> clients = bySession.get(sessionId);
        return clients == null ? 0 : clients.size();
    }

    private void drop(UUID sessionId, SseEmitter emitter) {
        List<SseEmitter> clients = bySession.get(sessionId);
        if (clients == null) {
            return;
        }
        clients.remove(emitter);
        if (clients.isEmpty()) {
            bySession.remove(sessionId, clients);
        }
    }
}
