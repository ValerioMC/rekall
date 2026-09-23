package dev.rekall.claude.queue;

import dev.rekall.claude.PtyTerminalManager;
import dev.rekall.common.ConflictException;
import dev.rekall.common.NotFoundException;
import dev.rekall.domain.RunQueue;
import dev.rekall.domain.RunQueueItem;
import dev.rekall.domain.RunQueueItemState;
import dev.rekall.domain.RunQueueState;
import dev.rekall.domain.Task;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.RunQueueItemRepository;
import dev.rekall.domain.repository.RunQueueRepository;
import dev.rekall.domain.repository.TaskRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;

/**
 * The whole write path of the run queue: what the console edits (the items, their order, the
 * settings, start and stop) and the moves {@link RunQueueRunner} makes as it works through it.
 * Every write publishes the resulting {@link RunQueueView} as a {@link RunQueueChangedEvent}.
 *
 * <p>It lives in {@code rekall-claude}, not {@code rekall-service}, because it is a console write:
 * the MCP module must not be able to reach it.
 *
 * <p>The rules it keeps: a task is queued once while it waits or runs; the running item is not
 * removed or moved, it is stopped; a queue with nothing waiting does not start; a start time in
 * the past is refused rather than run at once; the settings apply from the next session opened
 * and the next ceiling check.
 */
@Service
public class RunQueueService {

    static final int CEILING_MIN = 1;
    static final int CEILING_MAX = 100;
    /** How far in the past a start time may be and still count as "now": a form submitted slowly. */
    private static final Duration START_SLACK = Duration.ofMinutes(1);

    private final RunQueueRepository queues;
    private final RunQueueItemRepository items;
    private final TaskRepository tasks;
    private final ApplicationEventPublisher events;
    private final Clock clock;

    @Autowired
    public RunQueueService(
            RunQueueRepository queues, RunQueueItemRepository items, TaskRepository tasks,
            ApplicationEventPublisher events) {
        this(queues, items, tasks, events, Clock.systemUTC());
    }

    RunQueueService(
            RunQueueRepository queues, RunQueueItemRepository items, TaskRepository tasks,
            ApplicationEventPublisher events, Clock clock) {
        this.queues = queues;
        this.items = items;
        this.tasks = tasks;
        this.events = events;
        this.clock = clock;
    }

    // ---------------------------------------------------------------- console

    @Transactional
    public RunQueueView view() {
        return RunQueueView.of(queue(), ordered());
    }

    @Transactional
    public RunQueueView updateSettings(Integer ceilingPercent, boolean skipPermissions, String model, String effort) {
        if (ceilingPercent != null && (ceilingPercent < CEILING_MIN || ceilingPercent > CEILING_MAX)) {
            throw new IllegalArgumentException(
                    "The ceiling is a usage percentage between %d and %d, or none.".formatted(CEILING_MIN, CEILING_MAX));
        }
        RunQueue queue = queue();
        queue.setCeilingPercent(ceilingPercent);
        queue.setSkipPermissions(skipPermissions);
        queue.setModel(choice(model, PtyTerminalManager.MODEL_ALIASES, "model"));
        queue.setEffort(choice(effort, PtyTerminalManager.EFFORT_LEVELS, "effort"));
        return publish(queue);
    }

    @Transactional
    public RunQueueView add(UUID taskId) {
        Task task = tasks.findById(taskId)
                .orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));
        List<RunQueueItem> current = ordered();
        boolean waiting = current.stream()
                .anyMatch(item -> item.getTask().getId().equals(taskId) && !item.getState().settled());
        if (waiting) {
            throw new ConflictException("'%s' is already in the queue.".formatted(task.getTitle()));
        }
        items.save(new RunQueueItem(task, current.size()));
        return publish(queue());
    }

    @Transactional
    public RunQueueView remove(UUID itemId) {
        RunQueueItem item = requireItem(itemId);
        if (item.getState() == RunQueueItemState.RUNNING) {
            throw new ConflictException(
                    "'%s' is running. Stop the queue to take it off.".formatted(item.getTask().getTitle()));
        }
        items.delete(item);
        items.flush();
        renumber(ordered());
        return publish(queue());
    }

    /**
     * Move a waiting item to {@code index} among the waiting items. Settled and running items keep
     * their places ahead of them; only the order still to run changes.
     */
    @Transactional
    public RunQueueView move(UUID itemId, int index) {
        RunQueueItem item = requireItem(itemId);
        if (item.getState() != RunQueueItemState.QUEUED) {
            throw new ConflictException("Only a task still waiting its turn can be moved.");
        }
        List<RunQueueItem> all = ordered();
        List<RunQueueItem> waiting = new ArrayList<>(all.stream()
                .filter(candidate -> candidate.getState() == RunQueueItemState.QUEUED)
                .toList());
        waiting.remove(item);
        waiting.add(Math.clamp(index, 0, waiting.size()), item);
        List<RunQueueItem> reordered = new ArrayList<>(all.stream()
                .filter(candidate -> candidate.getState() != RunQueueItemState.QUEUED)
                .toList());
        reordered.addAll(waiting);
        renumber(reordered);
        return publish(queue());
    }

    /** Take every finished, skipped and failed item off the list. */
    @Transactional
    public RunQueueView clearSettled() {
        List<RunQueueItem> all = ordered();
        List<RunQueueItem> settled = all.stream().filter(item -> item.getState().settled()).toList();
        items.deleteAll(settled);
        items.flush();
        renumber(all.stream().filter(item -> !item.getState().settled()).toList());
        return publish(queue());
    }

    /**
     * Arm the queue: at once when {@code startAt} is null or already here, otherwise at
     * {@code startAt}. A scheduled queue can be started again to move or drop its time; a running
     * or holding one cannot.
     */
    @Transactional
    public RunQueueView start(Instant startAt) {
        RunQueue queue = queue();
        if (queue.getState() == RunQueueState.RUNNING || queue.getState() == RunQueueState.HOLDING) {
            throw new ConflictException("The queue is already running. Stop it first to start it again.");
        }
        boolean anyWaiting = ordered().stream().anyMatch(item -> item.getState() == RunQueueItemState.QUEUED);
        if (!anyWaiting) {
            throw new ConflictException("Nothing is waiting in the queue. Add a task first.");
        }
        Instant now = clock.instant();
        if (startAt != null && startAt.isBefore(now.minus(START_SLACK))) {
            throw new IllegalArgumentException("That start time has already passed. Pick a later one, or start now.");
        }
        if (startAt == null || !startAt.isAfter(now)) {
            queue.moveTo(RunQueueState.RUNNING);
        } else {
            queue.moveTo(RunQueueState.SCHEDULED);
            queue.setStartAt(startAt);
        }
        return publish(queue);
    }

    /** Disarm the queue. A running item goes back to waiting, at the head, with the reason. */
    @Transactional
    public RunQueueView stop(String reason) {
        RunQueue queue = queue();
        queue.moveTo(RunQueueState.IDLE);
        ordered().stream()
                .filter(item -> item.getState() == RunQueueItemState.RUNNING)
                .forEach(item -> item.moveTo(RunQueueItemState.QUEUED, reason));
        return publish(queue);
    }

    // ---------------------------------------------------------------- runner

    /** Settings and state as the runner reads them at a decision. */
    @Transactional(readOnly = true)
    public Snapshot snapshot() {
        RunQueue queue = queues.findFirstByOrderByCreatedAtAsc().orElse(null);
        if (queue == null) {
            return new Snapshot(RunQueueState.IDLE, null, null, null, false, null, null);
        }
        return new Snapshot(queue.getState(), queue.getStartAt(), queue.getHoldUntil(), queue.getCeilingPercent(),
                queue.isSkipPermissions(), queue.getModel(), queue.getEffort());
    }

    /** The first item still waiting, if any: the next one to run. */
    @Transactional(readOnly = true)
    public Optional<RunQueueView.Item> nextWaiting() {
        return ordered().stream()
                .filter(item -> item.getState() == RunQueueItemState.QUEUED)
                .findFirst()
                .map(RunQueueView.Item::of);
    }

    /** {@code SCHEDULED} or {@code HOLDING} to {@code RUNNING}: the time came, or the window reset. */
    @Transactional
    public void resume() {
        RunQueue queue = queue();
        if (queue.getState() == RunQueueState.SCHEDULED || queue.getState() == RunQueueState.HOLDING) {
            queue.moveTo(RunQueueState.RUNNING);
            publish(queue);
        }
    }

    @Transactional
    public void hold(Instant until, String reason) {
        RunQueue queue = queue();
        if (!queue.getState().armed()) {
            return;
        }
        queue.moveTo(RunQueueState.HOLDING);
        queue.setHoldUntil(until);
        queue.setHoldReason(reason);
        publish(queue);
    }

    /** The last waiting item has had its turn: back to {@code IDLE}. */
    @Transactional
    public void finish() {
        RunQueue queue = queue();
        if (queue.getState().armed()) {
            queue.moveTo(RunQueueState.IDLE);
            publish(queue);
        }
    }

    /** Record an item's move. An item deleted in the meantime (its task was) is ignored. */
    @Transactional
    public void markItem(UUID itemId, RunQueueItemState state, String reason) {
        items.findById(itemId).ifPresent(item -> {
            item.moveTo(state, reason);
            publish(queue());
        });
    }

    /**
     * After a restart no session survives, so an item recorded as running is waiting again. The
     * queue keeps its state: an armed queue picks the item back up on the runner's first tick.
     */
    @Transactional
    public void recoverAfterRestart() {
        List<RunQueueItem> orphaned = ordered().stream()
                .filter(item -> item.getState() == RunQueueItemState.RUNNING)
                .toList();
        orphaned.forEach(item -> item.moveTo(RunQueueItemState.QUEUED,
                "Rekall restarted while this ran; it picks up at the next open step."));
        if (!orphaned.isEmpty()) {
            publish(queue());
        }
    }

    /** What the runner needs to decide, read in one transaction. */
    public record Snapshot(
            RunQueueState state,
            Instant startAt,
            Instant holdUntil,
            Integer ceilingPercent,
            boolean skipPermissions,
            String model,
            String effort) {
    }

    // ---------------------------------------------------------------- helpers

    private RunQueue queue() {
        return queues.findFirstByOrderByCreatedAtAsc().orElseGet(() -> queues.saveAndFlush(new RunQueue()));
    }

    private List<RunQueueItem> ordered() {
        return items.findAllByOrderByPositionAsc();
    }

    private RunQueueItem requireItem(UUID itemId) {
        return items.findById(itemId)
                .orElseThrow(() -> new NotFoundException("No queued task with id " + itemId));
    }

    private void renumber(List<RunQueueItem> ordered) {
        for (int at = 0; at < ordered.size(); at++) {
            ordered.get(at).setPosition(at);
        }
        items.flush();
    }

    /**
     * Stamp, save and announce. The stamp moves on every write, an item's included, because the
     * console orders what it receives by it and drops anything older than what it holds.
     */
    private RunQueueView publish(RunQueue queue) {
        queue.setUpdatedAt(clock.instant());
        queues.saveAndFlush(queue);
        RunQueueView view = RunQueueView.of(queue, ordered());
        events.publishEvent(new RunQueueChangedEvent(view));
        return view;
    }

    /** Blank or {@code default} is the account's own setting (null); anything else must be offered. */
    private static String choice(String value, Set<String> allowed, String what) {
        if (value == null || value.isBlank() || value.strip().equalsIgnoreCase("default")) {
            return null;
        }
        String normalised = value.strip().toLowerCase(Locale.ROOT);
        if (!allowed.contains(normalised)) {
            throw new IllegalArgumentException("'%s' is not a %s Claude Code accepts.".formatted(value, what));
        }
        return normalised;
    }
}
