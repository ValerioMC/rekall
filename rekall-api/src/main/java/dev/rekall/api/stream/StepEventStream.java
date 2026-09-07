package dev.rekall.api.stream;

import dev.rekall.domain.step.StepStreamEvent;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import org.springframework.transaction.event.TransactionPhase;
import org.springframework.transaction.event.TransactionalEventListener;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.io.IOException;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;

/**
 * The open console windows, and one checklist change fanned out to all of them.
 *
 * <p>This is the read side of the live-step loop. A session moves a step over MCP,
 * {@code TaskStepService} publishes a {@link StepStreamEvent} after the write commits, and this
 * pushes it to every browser holding a connection so the animation reacts without a reload.
 *
 * <p>Held after commit, not on publish: a window must never be told about a change that then
 * rolls back. Sends are best-effort; a connection that has gone away is dropped on the next
 * failure rather than tracked.
 */
@Component
@Slf4j
public class StepEventStream {

    /** No timeout: a console is meant to hold this open for as long as it is on screen. */
    private static final long NO_TIMEOUT = 0L;

    private final List<SseEmitter> clients = new CopyOnWriteArrayList<>();

    /**
     * A new console connection. It is handed back to the controller to return, and taken off the
     * list the moment it completes, times out or errors.
     */
    public SseEmitter open() {
        SseEmitter emitter = new SseEmitter(NO_TIMEOUT);
        emitter.onCompletion(() -> clients.remove(emitter));
        emitter.onTimeout(() -> clients.remove(emitter));
        emitter.onError(error -> clients.remove(emitter));
        clients.add(emitter);
        try {
            emitter.send(SseEmitter.event().name("open").data("ready"));
        } catch (IOException e) {
            clients.remove(emitter);
        }
        return emitter;
    }

    @TransactionalEventListener(phase = TransactionPhase.AFTER_COMMIT, fallbackExecution = true)
    public void onStepChange(StepStreamEvent event) {
        for (SseEmitter emitter : clients) {
            try {
                emitter.send(SseEmitter.event().name("steps").data(event));
            } catch (IOException | IllegalStateException e) {
                // The window is gone or the response is already closed. Drop it and move on.
                clients.remove(emitter);
            }
        }
    }

    /** How many consoles are listening. For the health of the feature, and for tests. */
    public int clientCount() {
        return clients.size();
    }
}
