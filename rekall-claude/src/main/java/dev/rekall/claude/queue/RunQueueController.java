package dev.rekall.claude.queue;

import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.time.Instant;
import java.util.UUID;

/**
 * The console's handle on the run queue. Every call answers with the whole queue as it stands, the
 * same {@link RunQueueView} the {@code run-queue} SSE event carries, so the console has one shape
 * to draw from whichever arrives first.
 */
@RestController
@RequestMapping("/api/run-queue")
@RequiredArgsConstructor
public class RunQueueController {

    private final RunQueueService queue;
    private final RunQueueRunner runner;

    /** {@code ceilingPercent} null is no ceiling; {@code model} and {@code effort} blank are the account's. */
    public record SettingsRequest(Integer ceilingPercent, Boolean skipPermissions, String model, String effort) {
    }

    public record AddRequest(UUID taskId) {
    }

    public record MoveRequest(Integer index) {
    }

    /** {@code startAt} null starts at once. */
    public record StartRequest(Instant startAt) {
    }

    @GetMapping
    public RunQueueView view() {
        return queue.view();
    }

    @PutMapping("/settings")
    public RunQueueView settings(@RequestBody SettingsRequest request) {
        return queue.updateSettings(
                request.ceilingPercent(), Boolean.TRUE.equals(request.skipPermissions()),
                request.model(), request.effort());
    }

    @PostMapping("/items")
    public RunQueueView add(@RequestBody AddRequest request) {
        if (request.taskId() == null) {
            throw new IllegalArgumentException("Name the task to queue.");
        }
        RunQueueView added = queue.add(request.taskId());
        runner.nudge();
        return added;
    }

    @DeleteMapping("/items/{itemId}")
    public RunQueueView remove(@PathVariable UUID itemId) {
        return queue.remove(itemId);
    }

    @PutMapping("/items/{itemId}/position")
    public RunQueueView move(@PathVariable UUID itemId, @RequestBody MoveRequest request) {
        if (request.index() == null) {
            throw new IllegalArgumentException("Say where the task goes.");
        }
        return queue.move(itemId, request.index());
    }

    @PostMapping("/clear")
    public RunQueueView clearSettled() {
        return queue.clearSettled();
    }

    @PostMapping("/start")
    public RunQueueView start(@RequestBody(required = false) StartRequest request) {
        return runner.start(request == null ? null : request.startAt());
    }

    @PostMapping("/stop")
    public RunQueueView stop() {
        return runner.stop();
    }
}
