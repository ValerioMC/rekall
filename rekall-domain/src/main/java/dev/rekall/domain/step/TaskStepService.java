package dev.rekall.domain.step;

import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStep;
import dev.rekall.domain.TaskStepState;
import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TaskStepRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

/**
 * The checklist under a task: what it is made of, in order, and where each part has got to.
 *
 * <p>Two ways in and one of them is new. The console still owns the shape of the list, appends,
 * moves, deletes and, above all, the tick that marks a step {@link TaskStepState#DONE}. What a
 * session can now do, over MCP through {@link #transition}, is move a step to
 * {@link TaskStepState#RUNNING} and {@link TaskStepState#CLAIMED}, which is what lets it walk
 * its own checklist. It cannot reach {@link TaskStepState#DONE}: that is a person saying they
 * reviewed the work.
 *
 * <p>Every write publishes a {@link StepStreamEvent} so the open console windows update without
 * a reload. Positions are dense from zero, and every write that can leave a gap renumbers the
 * whole list.
 */
@Service
@RequiredArgsConstructor
public class TaskStepService {

    private final TaskRepository tasks;
    private final TaskStepRepository steps;
    private final ApplicationEventPublisher events;

    // ------------------------------------------------------------------ reading

    @Transactional(readOnly = true)
    public List<TaskStepView> findAll() {
        return steps.findAllByOrderByTaskIdAscPositionAsc().stream().map(TaskStepView::of).toList();
    }

    @Transactional(readOnly = true)
    public List<TaskStepView> findByTask(UUID taskId) {
        return steps.findByTaskIdOrderByPositionAsc(taskId).stream().map(TaskStepView::of).toList();
    }

    // ------------------------------------------------------------------ writing

    /**
     * Appends a step to the end of the task's list.
     *
     * <p>The end and nowhere else. A checklist is written in the order the work is thought of,
     * and a new item that landed in the middle because it sorted there would be a list nobody
     * could read back.
     */
    @Transactional
    public TaskStepView add(UUID taskId, String title, String bodyMarkdown) {
        Task task = tasks.findById(taskId)
                .orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));

        List<TaskStep> siblings = steps.findByTaskIdOrderByPositionAsc(taskId);
        TaskStep step = new TaskStep(task, validatedTitle(title), siblings.size());
        step.setBodyMarkdown(validatedBody(bodyMarkdown));
        task.getSteps().add(step);
        TaskStepView saved = TaskStepView.of(steps.saveAndFlush(step));
        publish(taskId);
        return saved;
    }

    /**
     * Changes one step from the console, one field at a time.
     *
     * <p>Null means "leave it alone" for every argument, so the checklist can be ticked from a
     * row that never loaded the detail behind it. {@code done} is the console's move onto and off
     * the {@link TaskStepState#DONE} end of the line: true accepts the work, false reopens it.
     * Clearing the detail is an empty string, which is stored as no detail rather than as an
     * empty paragraph.
     */
    @Transactional
    public TaskStepView edit(UUID id, String title, String bodyMarkdown, Boolean done) {
        TaskStep step = require(id);
        if (title != null) {
            step.setTitle(validatedTitle(title));
        }
        if (bodyMarkdown != null) {
            step.setBodyMarkdown(validatedBody(bodyMarkdown));
        }
        if (done != null) {
            step.markState(done ? TaskStepState.DONE : TaskStepState.OPEN);
        }
        TaskStepView saved = TaskStepView.of(steps.saveAndFlush(step));
        publish(step.getTask().getId());
        return saved;
    }

    /**
     * A session moving a step along the line, over MCP.
     *
     * <p>{@code target} is {@link TaskStepState#RUNNING} when the work starts,
     * {@link TaskStepState#CLAIMED} when the session is finished with it, or
     * {@link TaskStepState#OPEN} to put back a step it has to abandon. {@link TaskStepState#DONE}
     * is refused: a session claiming its own work is accepted is the one thing this split exists
     * to prevent. A step a person has already accepted is refused too, in either direction:
     * reopening it is a console decision.
     *
     * @param stepRef the step's one-based position in the list, or its exact title
     */
    @Transactional
    public TaskStepView transition(String projectLabel, String taskLabel, String stepRef, TaskStepState target) {
        if (target == TaskStepState.DONE) {
            throw new IllegalArgumentException(
                    "A step is marked done in the console, by the person who reviewed the work. "
                            + "A session can take it as far as `claimed`.");
        }
        Task task = resolveTask(projectLabel, taskLabel);
        TaskStep step = resolveStep(task.getId(), stepRef);
        if (step.getState() == TaskStepState.DONE) {
            throw new IllegalArgumentException(
                    "'%s' is already done. Reopen it in the console if that was wrong.".formatted(step.getTitle()));
        }
        step.markState(target);
        TaskStepView saved = TaskStepView.of(steps.saveAndFlush(step));
        publish(task.getId());
        return saved;
    }

    /**
     * Moves a step to a position in its own task's list, and renumbers what it displaced.
     *
     * <p>A position outside the list is clamped rather than refused: the caller is a row being
     * dragged or a key being held, and both mean "as far as it goes" at the edges.
     */
    @Transactional
    public List<TaskStepView> move(UUID id, int position) {
        TaskStep step = require(id);
        UUID taskId = step.getTask().getId();

        List<TaskStep> ordered = new ArrayList<>(steps.findByTaskIdOrderByPositionAsc(taskId));
        ordered.removeIf(candidate -> candidate.getId().equals(id));
        ordered.add(Math.clamp(position, 0, ordered.size()), step);
        renumber(ordered);

        steps.flush();
        publish(taskId);
        return ordered.stream().map(TaskStepView::of).toList();
    }

    /** Removing one step. The rest of the list closes the gap it leaves. */
    @Transactional
    public void delete(UUID id) {
        steps.findById(id).ifPresent(step -> {
            UUID taskId = step.getTask().getId();
            step.getTask().getSteps().remove(step);
            steps.delete(step);
            steps.flush();
            renumber(steps.findByTaskIdOrderByPositionAsc(taskId));
            publish(taskId);
        });
    }

    // ------------------------------------------------------------------ stream

    /**
     * Announces the task's checklist as it stands now.
     *
     * <p>The whole list rather than the one row that moved: a move renumbers all of them, and a
     * client that replaces what it holds for the task cannot end up half-updated.
     */
    private void publish(UUID taskId) {
        List<TaskStepView> current = steps.findByTaskIdOrderByPositionAsc(taskId).stream()
                .map(TaskStepView::of).toList();
        events.publishEvent(new StepStreamEvent(taskId, current));
    }

    // ------------------------------------------------------------------ addressing

    /**
     * The task an anchor names, with the rules {@code WrapupService} resolves by: a bare label
     * that two projects share is reported, never guessed.
     */
    private Task resolveTask(String projectLabel, String taskLabel) {
        if (projectLabel != null) {
            return tasks.findByProjectLabelIgnoreCaseAndLabelIgnoreCase(projectLabel, taskLabel)
                    .orElseThrow(() -> new UnknownAnchorException(
                            "No task '%s' on project '%s'".formatted(taskLabel, projectLabel)));
        }
        List<Task> found = tasks.findByLabelIgnoreCase(taskLabel);
        if (found.isEmpty()) {
            throw new UnknownAnchorException("No task matches '%s'".formatted(taskLabel));
        }
        if (found.size() > 1) {
            throw new AmbiguousAnchorException(
                    taskLabel, found.stream().map(task -> "project:" + task.getProject().getLabel()).toList());
        }
        return found.getFirst();
    }

    /**
     * Which step on the task the caller means: its one-based position, or its exact title.
     *
     * <p>A number is how a session refers to "step 3" after reading the checklist; a title is
     * the fallback for when it has the words but not the count. Anything that resolves to no
     * step, or a title that matches none, is refused with what the list actually holds.
     */
    private TaskStep resolveStep(UUID taskId, String ref) {
        List<TaskStep> ordered = steps.findByTaskIdOrderByPositionAsc(taskId);
        if (ordered.isEmpty()) {
            throw new IllegalArgumentException("That task has no steps to move.");
        }
        String trimmed = ref == null ? "" : ref.strip();

        if (trimmed.matches("\\d+")) {
            int oneBased = Integer.parseInt(trimmed);
            if (oneBased < 1 || oneBased > ordered.size()) {
                throw new IllegalArgumentException(
                        "There is no step %d. The checklist has %d.".formatted(oneBased, ordered.size()));
            }
            return ordered.get(oneBased - 1);
        }

        List<TaskStep> byTitle = ordered.stream()
                .filter(step -> step.getTitle().strip().equalsIgnoreCase(trimmed))
                .toList();
        if (byTitle.size() == 1) {
            return byTitle.getFirst();
        }
        String titles = ordered.stream()
                .map(step -> "%d. %s".formatted(ordered.indexOf(step) + 1, step.getTitle()))
                .reduce((a, b) -> a + "\n" + b)
                .orElse("");
        throw new IllegalArgumentException(
                "No step matches '%s'. Pass its number or its exact title:\n%s".formatted(ref, titles));
    }

    // ------------------------------------------------------------------ checks

    private void renumber(List<TaskStep> ordered) {
        for (int at = 0; at < ordered.size(); at++) {
            ordered.get(at).setPosition(at);
        }
    }

    private TaskStep require(UUID id) {
        return steps.findById(id).orElseThrow(() -> new UnknownAnchorException("No step with id " + id));
    }

    private String validatedTitle(String title) {
        if (title == null || title.isBlank()) {
            throw new IllegalArgumentException("A step needs a title. To remove one, delete it.");
        }
        String text = title.strip();
        if (text.length() > 200) {
            throw new IllegalArgumentException(
                    "A step's title is capped at 200 characters and this one is %d. What it has to satisfy "
                            .formatted(text.length()) + "goes in the detail below it.");
        }
        return text;
    }

    /**
     * The detail, or none.
     *
     * <p>Capped where the wrapup is, and refused here rather than at the column so the message
     * says what to do about it: a step whose detail runs past a screen is a task, and the model
     * already has a level for that.
     */
    private String validatedBody(String bodyMarkdown) {
        if (bodyMarkdown == null || bodyMarkdown.isBlank()) {
            return null;
        }
        String text = bodyMarkdown.strip();
        if (text.length() > TaskStep.MAX_CHARACTERS) {
            throw new IllegalArgumentException(
                    "A step's detail is capped at %d characters and this one is %d. A step that long is a "
                            .formatted(TaskStep.MAX_CHARACTERS, text.length())
                            + "task of its own; split it, or move the detail into a note.");
        }
        return text;
    }
}
