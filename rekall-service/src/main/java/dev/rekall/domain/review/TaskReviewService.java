package dev.rekall.domain.review;

import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStepState;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.TaskRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.UUID;

/**
 * The task-scoped mirror of {@code TaskStepService}: walks a stepless task along
 * {@code OPEN -> RUNNING -> CLAIMED -> DONE} and pushes every move onto the step SSE feed as a
 * {@link TaskReviewEvent}. {@link #sessionRunning} and {@link #claimedByWrapup} ride on existing
 * signals; {@link #accept} and {@link #sendBack} are the console's alone, with no MCP tool. Every
 * method is inert once the task has a checklist.
 */
@Service
@RequiredArgsConstructor
public class TaskReviewService {

    private final TaskRepository tasks;
    private final ApplicationEventPublisher events;

    /** Flip {@code OPEN <-> RUNNING} as a session on the task anchor comes and goes. Ambient: never blocks a claim. */
    @Transactional
    public void sessionRunning(UUID taskId, boolean running) {
        Task task = tasks.findById(taskId).orElse(null);
        if (task == null || !task.reviewActive()) {
            return;
        }
        if (running && task.getReviewState() == TaskStepState.OPEN) {
            task.markReviewState(TaskStepState.RUNNING);
            publish(task);
        } else if (!running && task.getReviewState() == TaskStepState.RUNNING) {
            task.markReviewState(TaskStepState.OPEN);
            publish(task);
        }
    }

    /** A Claude-authored wrapup advances {@code OPEN | RUNNING -> CLAIMED}. A hand-written one does not. */
    @Transactional
    public void claimedByWrapup(UUID taskId) {
        Task task = tasks.findById(taskId).orElse(null);
        if (task == null || !task.reviewActive()) {
            return;
        }
        TaskStepState state = task.getReviewState();
        if (state == TaskStepState.OPEN || state == TaskStepState.RUNNING) {
            task.markReviewState(TaskStepState.CLAIMED);
            publish(task);
        }
    }

    /** The console accepts the work: {@code -> DONE} from any state but {@code DONE} itself. */
    @Transactional
    public TaskReviewView accept(UUID taskId) {
        Task task = require(taskId);
        guardActive(task);
        if (task.getReviewState() == TaskStepState.DONE) {
            throw new IllegalArgumentException(
                    "This task is already accepted. Send it back to reopen it for another pass.");
        }
        task.markReviewState(TaskStepState.DONE);
        return publish(task);
    }

    /** The console sends the work back: {@code -> OPEN}, with an optional note; blank notes are dropped. */
    @Transactional
    public TaskReviewView sendBack(UUID taskId, String note) {
        Task task = require(taskId);
        guardActive(task);
        task.markReviewState(TaskStepState.OPEN);
        task.setReviewNote(note == null || note.isBlank() ? null : note.strip());
        return publish(task);
    }

    private void guardActive(Task task) {
        if (!task.reviewActive()) {
            throw new IllegalArgumentException(
                    "This task has a checklist, so its steps carry the review, not the task.");
        }
    }

    private Task require(UUID taskId) {
        return tasks.findById(taskId)
                .orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));
    }

    private TaskReviewView publish(Task task) {
        TaskReviewView view = TaskReviewView.of(task);
        events.publishEvent(new TaskReviewEvent(task.getId(), view));
        return view;
    }
}
