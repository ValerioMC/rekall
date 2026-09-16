package dev.rekall.api.stream;

import dev.rekall.domain.review.TaskReviewEvent;
import dev.rekall.domain.step.StepStreamEvent;
import dev.rekall.domain.wrapup.WrapupStreamEvent;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.ContextClosedEvent;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Component;
import org.springframework.transaction.event.TransactionPhase;
import org.springframework.transaction.event.TransactionalEventListener;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.io.IOException;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;

@Component
@Slf4j
public class StepEventStream {

    private static final long NO_TIMEOUT = 0L;

    private final List<SseEmitter> clients = new CopyOnWriteArrayList<>();

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
        dispatch("steps", event);
    }

    @TransactionalEventListener(phase = TransactionPhase.AFTER_COMMIT, fallbackExecution = true)
    public void onTaskReview(TaskReviewEvent event) {
        dispatch("task-review", event);
    }

    @TransactionalEventListener(phase = TransactionPhase.AFTER_COMMIT, fallbackExecution = true)
    public void onWrapupChange(WrapupStreamEvent event) {
        dispatch("wrapup", event);
    }

    private void dispatch(String name, Object payload) {
        for (SseEmitter emitter : clients) {
            try {
                emitter.send(SseEmitter.event().name(name).data(payload));
            } catch (IOException | IllegalStateException e) {
                clients.remove(emitter);
            }
        }
    }

    // Without this, graceful shutdown blocks on every open feed until the per-phase timeout elapses.
    @EventListener(ContextClosedEvent.class)
    public void releaseOnShutdown() {
        for (SseEmitter emitter : clients) {
            try {
                emitter.complete();
            } catch (RuntimeException e) {
                // Already closed from the other end; the onCompletion/onError callback has it.
            }
        }
        clients.clear();
    }

    public int clientCount() {
        return clients.size();
    }
}
