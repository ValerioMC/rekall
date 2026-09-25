package dev.rekall.claude.queue;

import dev.rekall.claude.ClaudeUsageService;
import dev.rekall.claude.ClaudeUsageView;
import dev.rekall.claude.PtyTerminalManager;
import dev.rekall.claude.TerminalApiDtos.TerminalView;
import dev.rekall.claude.TerminalEndedEvent;
import dev.rekall.claude.TerminalMode;
import dev.rekall.claude.queue.RunQueueService.Snapshot;
import dev.rekall.claude.queue.UsageCeiling.Verdict;
import dev.rekall.common.ConflictException;
import dev.rekall.domain.RunQueueItemState;
import dev.rekall.domain.RunQueueState;
import dev.rekall.domain.claude.TaskWorkService;
import dev.rekall.domain.claude.TaskWorkService.Remaining;
import dev.rekall.domain.claude.TaskWorkService.TaskWork;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.review.TaskReviewEvent;
import dev.rekall.domain.step.StepStreamEvent;
import dev.rekall.domain.step.TaskStepService;
import jakarta.annotation.PostConstruct;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.event.ContextClosedEvent;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Component;
import org.springframework.transaction.event.TransactionPhase;
import org.springframework.transaction.event.TransactionalEventListener;

import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.Callable;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

/**
 * Works through the run queue, one task at a time, each in a {@code claude} terminal of its own.
 *
 * <p>Every decision runs on one thread, so the runner's memory of the item in hand needs no lock:
 * a clock tick every {@code rekall.run-queue.tick-seconds}, and a nudge from every step move,
 * review move and terminal end on the task it is running. The line for each item:
 *
 * <ol>
 *   <li>Take the first waiting item. None left: the queue goes idle.</li>
 *   <li>Check the usage ceiling. At or over it: hold until the window resets.</li>
 *   <li>Nothing open on the task (all claimed or accepted): skip it. A terminal someone else has
 *       open on it, a missing folder or CLI: fail it with the reason, and go on.</li>
 *   <li>Open a terminal on the task, which types {@code /rk} with its anchors. The session works
 *       the open steps in order, claiming each.</li>
 *   <li>Every claim is a step boundary, and the ceiling is checked again there. Over it: the
 *       terminal closes, the item waits again at the head, and the queue holds. The next session
 *       picks up at the first open step, so no claimed work is repeated.</li>
 *   <li>Nothing left open: the item is finished, its terminal closed, and the next one starts.</li>
 *   <li>The terminal ends while work is still open: the item fails, and the next one starts.</li>
 * </ol>
 *
 * <p>A terminal is closed {@link #settleGrace} after the claim that ends its turn, not at once:
 * the claim is the session's last tool call, and the grace lets it print its closing lines. The
 * runner never interrupts a step: a session mid-step when usage crosses the ceiling finishes that
 * step first, which is why the ceiling sits below the account's limit rather than at it.
 */
@Component
@Slf4j
public class RunQueueRunner {

    private final RunQueueService queue;
    private final PtyTerminalManager terminals;
    private final ClaudeUsageService usage;
    private final TaskWorkService work;
    private final TaskStepService steps;
    private final Clock clock;
    private final Duration tick;
    private final Duration settleGrace;

    private ScheduledExecutorService executor;

    /** The item a session is working now. Read and written on {@link #executor} only. */
    private Active active;

    private static final class Active {
        private final UUID itemId;
        private final UUID taskId;
        private final UUID terminalId;
        private int settledSteps;
        /** Set once the runner has decided to close this terminal, so a late event changes nothing. */
        private boolean closing;

        private Active(UUID itemId, UUID taskId, UUID terminalId, int settledSteps) {
            this.itemId = itemId;
            this.taskId = taskId;
            this.terminalId = terminalId;
            this.settledSteps = settledSteps;
        }
    }

    @Autowired
    public RunQueueRunner(
            RunQueueService queue,
            PtyTerminalManager terminals,
            ClaudeUsageService usage,
            TaskWorkService work,
            TaskStepService steps,
            @Value("${rekall.run-queue.tick-seconds:20}") long tickSeconds,
            @Value("${rekall.run-queue.settle-grace-seconds:8}") long settleGraceSeconds) {
        this(queue, terminals, usage, work, steps, Clock.systemUTC(),
                Duration.ofSeconds(tickSeconds), Duration.ofSeconds(settleGraceSeconds));
    }

    RunQueueRunner(
            RunQueueService queue,
            PtyTerminalManager terminals,
            ClaudeUsageService usage,
            TaskWorkService work,
            TaskStepService steps,
            Clock clock,
            Duration tick,
            Duration settleGrace) {
        this.queue = queue;
        this.terminals = terminals;
        this.usage = usage;
        this.work = work;
        this.steps = steps;
        this.clock = clock;
        this.tick = tick;
        this.settleGrace = settleGrace;
    }

    // ---------------------------------------------------------------- lifecycle

    @PostConstruct
    void start() {
        executor = Executors.newSingleThreadScheduledExecutor(runnable -> {
            Thread thread = new Thread(runnable, "run-queue");
            thread.setDaemon(true);
            return thread;
        });
        executor.execute(this::recover);
        executor.scheduleWithFixedDelay(this::tick, tick.toMillis(), tick.toMillis(), TimeUnit.MILLISECONDS);
    }

    /** Stop deciding before the terminals go: {@link PtyTerminalManager} closes those itself. */
    @EventListener(ContextClosedEvent.class)
    void shutdown() {
        if (executor != null) {
            executor.shutdownNow();
        }
    }

    // ---------------------------------------------------------------- console

    /** Arm the queue and look at it at once rather than on the next tick. */
    public RunQueueView start(Instant startAt) {
        RunQueueView started = queue.start(startAt);
        nudge();
        return started;
    }

    /** Disarm the queue, closing the session it has open. Done on the runner's thread, and waited for. */
    public RunQueueView stop() {
        return onRunner(() -> {
            if (active != null) {
                Active stopping = active;
                active = null;
                closeSession(stopping, "Stopped from the console.");
            }
            return queue.stop("Stopped from the console; it picks up at the next open step.");
        });
    }

    /** Take a look now: the console changed something the next tick would otherwise wait for. */
    public void nudge() {
        submit(this::tick);
    }

    // ---------------------------------------------------------------- signals

    @TransactionalEventListener(phase = TransactionPhase.AFTER_COMMIT, fallbackExecution = true)
    public void onSteps(StepStreamEvent event) {
        submit(() -> review(event.taskId()));
    }

    @TransactionalEventListener(phase = TransactionPhase.AFTER_COMMIT, fallbackExecution = true)
    public void onTaskReview(TaskReviewEvent event) {
        submit(() -> review(event.taskId()));
    }

    @EventListener
    public void onTerminalEnded(TerminalEndedEvent event) {
        submit(() -> sessionEnded(event.terminalId()));
    }

    // ---------------------------------------------------------------- decisions (runner thread)

    private void recover() {
        guarded("recover", queue::recoverAfterRestart);
    }

    void tick() {
        guarded("tick", () -> {
            Snapshot snapshot = queue.snapshot();
            Instant now = clock.instant();
            switch (snapshot.state()) {
                case IDLE -> {
                    return;
                }
                case SCHEDULED -> {
                    if (snapshot.startAt() != null && now.isBefore(snapshot.startAt())) {
                        return;
                    }
                    log.info("Run queue: start time reached");
                    queue.resume();
                }
                case HOLDING -> {
                    if (snapshot.holdUntil() != null && now.isBefore(snapshot.holdUntil())) {
                        return;
                    }
                    log.info("Run queue: hold over, taking a fresh usage reading");
                    usage.refresh();
                    queue.resume();
                }
                case RUNNING -> {
                    // Nothing to change before looking at the item in hand.
                }
            }
            if (active != null) {
                if (!active.closing && !terminals.isLive(active.terminalId)) {
                    sessionEnded(active.terminalId);
                } else {
                    review(active.taskId);
                }
                return;
            }
            advance();
        });
    }

    /** Start the next waiting item, skipping and failing past the ones that cannot run. */
    private void advance() {
        while (active == null) {
            Snapshot snapshot = queue.snapshot();
            if (snapshot.state() != RunQueueState.RUNNING) {
                return;
            }
            Optional<RunQueueView.Item> next = queue.nextWaiting();
            if (next.isEmpty()) {
                log.info("Run queue: every queued task has had its turn");
                queue.finish();
                return;
            }
            Verdict verdict = ceiling(snapshot);
            if (!verdict.clear()) {
                log.info("Run queue: holding until {} ({})", verdict.resumeAt(), verdict.reason());
                queue.hold(verdict.resumeAt(), verdict.reason());
                return;
            }
            launch(next.get(), snapshot);
        }
    }

    private void launch(RunQueueView.Item item, Snapshot snapshot) {
        TaskWork before;
        try {
            before = work.read(item.taskId());
        } catch (UnknownAnchorException gone) {
            queue.markItem(item.id(), RunQueueItemState.FAILED, "The task no longer exists.");
            return;
        }
        if (!before.hasWork()) {
            queue.markItem(item.id(), RunQueueItemState.SKIPPED, nothingOpen(before.remaining()));
            return;
        }
        if (terminals.hasLiveTerminalFor(item.taskId())) {
            queue.markItem(item.id(), RunQueueItemState.FAILED,
                    "A terminal was already open on this task. Close it and queue the task again.");
            return;
        }
        TerminalView opened;
        try {
            opened = terminals.open(
                    item.taskId(), null, snapshot.skipPermissions(), snapshot.model(), snapshot.effort(), TerminalMode.WORK);
        } catch (ConflictException | IllegalArgumentException | UnknownAnchorException refused) {
            queue.markItem(item.id(), RunQueueItemState.FAILED, refused.getMessage());
            return;
        }
        active = new Active(item.id(), item.taskId(), opened.id(), before.settledSteps());
        queue.markItem(item.id(), RunQueueItemState.RUNNING, null);
        log.info("Run queue: running {} in terminal {}", item.anchor(), opened.id());
    }

    /** A step or the review line moved on the task in hand: finished, or a boundary to check. */
    private void review(UUID taskId) {
        Active current = active;
        if (current == null || current.closing || !current.taskId.equals(taskId)) {
            return;
        }
        TaskWork now;
        try {
            now = work.read(taskId);
        } catch (UnknownAnchorException gone) {
            active = null;
            closeSession(current, "The task was deleted.");
            advance();
            return;
        }
        if (!now.hasWork()) {
            current.closing = true;
            later(() -> settle(current, RunQueueItemState.FINISHED, finished(now.remaining())));
            return;
        }
        if (now.settledSteps() > current.settledSteps) {
            current.settledSteps = now.settledSteps();
            Verdict verdict = ceiling(queue.snapshot());
            if (!verdict.clear()) {
                current.closing = true;
                later(() -> pause(current, verdict));
            }
        }
    }

    /** The item is done with; close its terminal, record the outcome, and go on to the next. */
    private void settle(Active current, RunQueueItemState outcome, String reason) {
        if (active != current) {
            return;
        }
        active = null;
        closeSession(current, "The run queue moved on to the next task.");
        queue.markItem(current.itemId, outcome, reason);
        advance();
    }

    /** At the ceiling on a step boundary: close the session, put the item back at the head, hold. */
    private void pause(Active current, Verdict verdict) {
        if (active != current) {
            return;
        }
        active = null;
        closeSession(current, "The run queue reached its usage ceiling.");
        queue.markItem(current.itemId, RunQueueItemState.QUEUED,
                "Paused at the usage ceiling; it picks up at the next open step.");
        queue.hold(verdict.resumeAt(), verdict.reason());
        log.info("Run queue: paused at the ceiling until {} ({})", verdict.resumeAt(), verdict.reason());
    }

    /** The terminal in hand went away without the runner closing it. */
    private void sessionEnded(UUID terminalId) {
        Active current = active;
        if (current == null || current.closing || !current.terminalId.equals(terminalId)) {
            return;
        }
        active = null;
        releaseRunningSteps(current.taskId);
        TaskWork after = readOrNull(current.taskId);
        if (after != null && !after.hasWork()) {
            queue.markItem(current.itemId, RunQueueItemState.FINISHED, finished(after.remaining()));
        } else {
            queue.markItem(current.itemId, RunQueueItemState.FAILED,
                    "The session ended before its work was claimed.");
        }
        advance();
    }

    // ---------------------------------------------------------------- helpers

    private Verdict ceiling(Snapshot snapshot) {
        if (snapshot.ceilingPercent() == null) {
            return Verdict.go();
        }
        ClaudeUsageView reading = usage.current();
        return UsageCeiling.check(reading, snapshot.ceilingPercent(), snapshot.model(), clock.instant());
    }

    /**
     * Close the item's terminal and put back any step its session had marked running: the step is
     * not finished, and the next session has to find it open to pick it up.
     */
    private void closeSession(Active current, String reason) {
        current.closing = true;
        try {
            terminals.close(current.terminalId, reason);
        } catch (RuntimeException closeFailed) {
            log.warn("Run queue: closing terminal {} failed: {}", current.terminalId, closeFailed.getMessage());
        }
        releaseRunningSteps(current.taskId);
    }

    private void releaseRunningSteps(UUID taskId) {
        TaskWork state = readOrNull(taskId);
        if (state != null) {
            state.runningStepIds().forEach(steps::releaseRunning);
        }
    }

    private TaskWork readOrNull(UUID taskId) {
        try {
            return work.read(taskId);
        } catch (UnknownAnchorException gone) {
            return null;
        }
    }

    private static String nothingOpen(Remaining remaining) {
        return remaining == Remaining.ACCEPTED
                ? "Nothing to run: all of it is already accepted."
                : "Nothing to run: all of it is claimed and waiting for review.";
    }

    private static String finished(Remaining remaining) {
        return remaining == Remaining.ACCEPTED
                ? "All of it accepted."
                : "All of it claimed and waiting for review.";
    }

    private void later(Runnable action) {
        if (executor == null || executor.isShutdown()) {
            return;
        }
        executor.schedule(() -> guarded("settle", action), settleGrace.toMillis(), TimeUnit.MILLISECONDS);
    }

    private void submit(Runnable action) {
        if (executor == null || executor.isShutdown()) {
            return;
        }
        try {
            executor.execute(() -> guarded("signal", action));
        } catch (java.util.concurrent.RejectedExecutionException shuttingDown) {
            log.debug("Run queue: signal dropped during shutdown");
        }
    }

    private <T> T onRunner(Callable<T> action) {
        try {
            return executor.submit(action).get(30, TimeUnit.SECONDS);
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            throw new ConflictException("Interrupted while stopping the queue.");
        } catch (TimeoutException slow) {
            throw new ConflictException("The queue did not stop in time. Try again.");
        } catch (ExecutionException failed) {
            if (failed.getCause() instanceof RuntimeException runtime) {
                throw runtime;
            }
            throw new IllegalStateException(failed.getCause());
        }
    }

    /** One failed decision is logged and the next tick tries again; it never kills the thread. */
    private void guarded(String what, Runnable action) {
        try {
            action.run();
        } catch (RuntimeException failed) {
            log.warn("Run queue: {} failed: {}", what, failed.getMessage(), failed);
        }
    }
}
