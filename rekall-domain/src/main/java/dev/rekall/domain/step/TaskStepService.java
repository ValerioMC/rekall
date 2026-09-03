package dev.rekall.domain.step;

import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStep;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TaskStepRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

/**
 * The checklist under a task: what it is made of, in order, and which parts are finished.
 *
 * <p>One writer, the console, the same trade {@code TimeEntryService} makes. A step is ticked
 * by the person who reviewed the work, never by the session that did it, so there is no MCP
 * path in and the ordering rules below are enforced here rather than by a constraint.
 *
 * <p>Positions are dense and start at zero. Every write that can leave a gap renumbers the
 * whole list, because the alternative is a sparse ordering that is correct until something
 * reads it as an index.
 */
@Service
@RequiredArgsConstructor
public class TaskStepService {

    private final TaskRepository tasks;
    private final TaskStepRepository steps;

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
        return TaskStepView.of(steps.saveAndFlush(step));
    }

    /**
     * Changes one step, one field at a time.
     *
     * <p>Null means "leave it alone" for every argument, so the checklist can be ticked from a
     * row that never loaded the detail behind it. Clearing the detail is an empty string, which
     * is stored as no detail rather than as an empty paragraph.
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
            step.markDone(done);
        }
        return TaskStepView.of(steps.saveAndFlush(step));
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
        });
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
