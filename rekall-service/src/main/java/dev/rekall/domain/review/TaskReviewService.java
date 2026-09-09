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
 * The task-scoped mirror of {@code TaskStepService}: it walks a stepless task
 * along {@code OPEN -> RUNNING -> CLAIMED -> DONE} and pushes every move onto the
 * step SSE feed as a {@link TaskReviewEvent}.
 *
 * <p>Three of the four moves ride on signals the rest of the system already
 * emits, so no new Claude-facing verb exists:
 *
 * <ul>
 *   <li>{@link #sessionRunning} is called by the process manager when a session
 *       attaches to or leaves the task anchor with no {@code step:} target.
 *   <li>{@link #claimedByWrapup} is called by the wrapup write path when the
 *       author is Claude.
 *   <li>{@link #accept} and {@link #sendBack} are the console's alone. There is
 *       no MCP tool that reaches them, the same way {@code TaskStepService}
 *       refuses {@code DONE}: a person accepts the work after looking at it.
 * </ul>
 *
 * <p>Every method is inert once the task has a checklist: the steps carry the
 * review then, and the task-level columns are kept but ignored.
 */
@Service
@RequiredArgsConstructor
public class TaskReviewService {

    private final TaskRepository tasks;
    private final ApplicationEventPublisher events;

    /**
     * Move a stepless task between {@code OPEN} and {@code RUNNING} as a session
     * on its anchor comes and goes. {@code running} is whether any live session
     * is currently attached to the task with no step target. Ambient: it never
     * blocks a claim or an accept, and a wrong flip is corrected by the next
     * signal.
     */
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

    /**
     * A Claude-authored wrapup is the deliverable of a stepless task, the way a
     * step's body is: writing one advances {@code OPEN | RUNNING -> CLAIMED}. A
     * hand-written wrapup does not, and neither does a task that already has a
     * checklist.
     */
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

    /**
     * The console accepts the work: {@code -> DONE}. Available from any state but
     * {@code DONE} itself, so a task with no wrapup can still be accepted. The
     * guard mirrors {@code TaskStepService.transition} refusing an already
     * accepted step.
     */
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

    /**
     * The console sends the work back: {@code -> OPEN}, with an optional note the
     * next session sees. Blank notes are dropped rather than stored.
     */
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
