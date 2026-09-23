package dev.rekall.claude.queue;

import dev.rekall.domain.RunQueue;
import dev.rekall.domain.RunQueueItem;
import dev.rekall.domain.RunQueueItemState;
import dev.rekall.domain.RunQueueState;
import dev.rekall.domain.Task;

import java.time.Instant;
import java.util.List;
import java.util.UUID;

/** The run queue as the console draws it: its state and settings, and every item in order. */
public record RunQueueView(
        RunQueueState state,
        Instant startAt,
        Integer ceilingPercent,
        boolean skipPermissions,
        String model,
        String effort,
        Instant holdUntil,
        String holdReason,
        List<Item> items,
        Instant updatedAt) {

    public RunQueueView {
        items = List.copyOf(items);
    }

    /** One queued task, with enough of the task to name it without a second request. */
    public record Item(
            UUID id,
            UUID taskId,
            String taskTitle,
            String taskLabel,
            String projectLabel,
            String anchor,
            int position,
            RunQueueItemState state,
            String detail,
            Instant startedAt,
            Instant finishedAt) {

        static Item of(RunQueueItem item) {
            Task task = item.getTask();
            String projectLabel = task.getProject().getLabel();
            return new Item(
                    item.getId(), task.getId(), task.getTitle(), task.getLabel(), projectLabel,
                    "project:%s task:%s".formatted(projectLabel, task.getLabel()),
                    item.getPosition(), item.getState(), item.getDetail(),
                    item.getStartedAt(), item.getFinishedAt());
        }
    }

    static RunQueueView of(RunQueue queue, List<RunQueueItem> items) {
        return new RunQueueView(
                queue.getState(), queue.getStartAt(), queue.getCeilingPercent(), queue.isSkipPermissions(),
                queue.getModel(), queue.getEffort(), queue.getHoldUntil(), queue.getHoldReason(),
                items.stream().map(Item::of).toList(), queue.getUpdatedAt());
    }
}
