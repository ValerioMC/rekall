package dev.rekall.domain.claude;

import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStep;
import dev.rekall.domain.TaskStepState;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TaskStepRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.UUID;

/**
 * Reads how much of a task is left for a session, the question the run queue asks before it opens
 * one and after every step moves. A task with a checklist is read from its non-draft steps; a
 * task without one from its review line. Read-only, and in the domain layer for the same reason
 * as {@link TerminalLaunchService}.
 */
@Service
@RequiredArgsConstructor
public class TaskWorkService {

    private final TaskRepository tasks;
    private final TaskStepRepository steps;

    @Transactional(readOnly = true)
    public TaskWork read(UUID taskId) {
        Task task = tasks.findById(taskId)
                .orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));
        List<TaskStep> checklist = steps.findByTaskIdOrderByPositionAsc(taskId).stream()
                .filter(step -> !step.getState().draft())
                .toList();
        if (checklist.isEmpty()) {
            return new TaskWork(remainingOf(task.getReviewState()), 0, List.of());
        }
        boolean workLeft = checklist.stream()
                .anyMatch(step -> step.getState() == TaskStepState.OPEN || step.getState().running());
        boolean allDone = checklist.stream().allMatch(step -> step.getState() == TaskStepState.DONE);
        Remaining remaining = workLeft ? Remaining.OPEN : allDone ? Remaining.ACCEPTED : Remaining.CLAIMED;
        int settled = (int) checklist.stream().filter(step -> step.getState().complete()).count();
        List<UUID> running = checklist.stream()
                .filter(step -> step.getState().running())
                .map(TaskStep::getId)
                .toList();
        return new TaskWork(remaining, settled, running);
    }

    private static Remaining remainingOf(TaskStepState reviewState) {
        return switch (reviewState) {
            case CLAIMED -> Remaining.CLAIMED;
            case DONE -> Remaining.ACCEPTED;
            default -> Remaining.OPEN;
        };
    }

    /** Whether a session has anything to do on the task, and if not, why not. */
    public enum Remaining {
        /** An open or running step, or a stepless task not yet claimed. */
        OPEN,
        /** Everything is claimed and at least part of it waits for review. */
        CLAIMED,
        /** Everything has been accepted. */
        ACCEPTED
    }

    /**
     * {@code settledSteps} counts the claimed and done steps, so a rise between two reads is a
     * claim; {@code runningStepIds} are the steps a session has marked running.
     */
    public record TaskWork(Remaining remaining, int settledSteps, List<UUID> runningStepIds) {

        public TaskWork {
            runningStepIds = List.copyOf(runningStepIds);
        }

        public boolean hasWork() {
            return remaining == Remaining.OPEN;
        }
    }
}
