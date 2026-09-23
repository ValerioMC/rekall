package dev.rekall.claude.queue;

import dev.rekall.api.stream.StepEventStream;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;
import org.springframework.transaction.event.TransactionPhase;
import org.springframework.transaction.event.TransactionalEventListener;

/** Carries every committed run queue change onto the console's one SSE feed, as {@code run-queue}. */
@Component
@RequiredArgsConstructor
class RunQueueBroadcaster {

    private final StepEventStream stream;

    @TransactionalEventListener(phase = TransactionPhase.AFTER_COMMIT, fallbackExecution = true)
    void onChange(RunQueueChangedEvent event) {
        stream.broadcast("run-queue", event.queue());
    }
}
