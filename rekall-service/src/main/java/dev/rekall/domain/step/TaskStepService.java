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
import java.util.Optional;
import java.util.UUID;

@Service
@RequiredArgsConstructor
public class TaskStepService {

    private final TaskRepository tasks;
    private final TaskStepRepository steps;
    private final ApplicationEventPublisher events;

    @Transactional(readOnly = true)
    public List<TaskStepView> findAll() {
        return steps.findAllByOrderByTaskIdAscPositionAsc().stream().map(TaskStepView::of).toList();
    }

    @Transactional(readOnly = true)
    public List<TaskStepView> findByTask(UUID taskId) {
        return steps.findByTaskIdOrderByPositionAsc(taskId).stream().map(TaskStepView::of).toList();
    }

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

    @Transactional
    public TaskStepView edit(UUID id, String title, String bodyMarkdown, Boolean done, Boolean draft) {
        TaskStep step = require(id);
        if (title != null) {
            step.setTitle(validatedTitle(title));
        }
        if (bodyMarkdown != null) {
            step.setBodyMarkdown(validatedBody(bodyMarkdown));
        }
        if (draft != null) {
            applyDraft(step, draft);
        }
        if (done != null) {
            step.markState(done ? TaskStepState.DONE : TaskStepState.OPEN);
        }
        UUID taskId = step.getTask().getId();
        steps.saveAndFlush(step);
        if (draft != null) {
            settleDraftsAtTail(taskId);
        }
        TaskStepView saved = TaskStepView.of(require(step.getId()));
        publish(taskId);
        return saved;
    }

    /**
     * Move a step between {@code DRAFT} and {@code OPEN}. Only those two states move; a step a
     * session has started, claimed or finished is refused.
     */
    private void applyDraft(TaskStep step, boolean draft) {
        TaskStepState current = step.getState();
        if (draft && current == TaskStepState.OPEN) {
            step.markState(TaskStepState.DRAFT);
        } else if (!draft && current == TaskStepState.DRAFT) {
            step.markState(TaskStepState.OPEN);
        } else if (draft != (current == TaskStepState.DRAFT)) {
            throw new IllegalArgumentException(
                    "'%s' is %s and cannot change its draft state. Only a step that is still draft or "
                            .formatted(step.getTitle(), current.name().toLowerCase())
                            + "open moves between the two.");
        }
    }

    @Transactional
    public TaskStepView transition(String projectLabel, String taskLabel, String stepRef, TaskStepState target) {
        if (target == TaskStepState.DONE) {
            throw new IllegalArgumentException(
                    "A step is marked done in the console, by the person who reviewed the work. "
                            + "A session can take it as far as `claimed`.");
        }
        Task task = resolveTask(projectLabel, taskLabel);
        TaskStep step = resolveStep(task.getId(), stepRef);
        if (step.getState() == TaskStepState.DRAFT) {
            throw new IllegalArgumentException(
                    "'%s' is still a draft. It is promoted to a workable step in the console, not "
                            .formatted(step.getTitle()) + "from a session.");
        }
        if (step.getState() == TaskStepState.DONE) {
            throw new IllegalArgumentException(
                    "'%s' is already done. Reopen it in the console if that was wrong.".formatted(step.getTitle()));
        }
        step.markState(target);
        TaskStepView saved = TaskStepView.of(steps.saveAndFlush(step));
        publish(task.getId());
        return saved;
    }

    @Transactional(readOnly = true)
    public Optional<String> titleOf(UUID stepId) {
        if (stepId == null) {
            return Optional.empty();
        }
        return steps.findById(stepId).map(TaskStep::getTitle);
    }

    /** Move a step {@code OPEN -> RUNNING} when a session opens on it. A no-op past {@code OPEN}. */
    @Transactional
    public void markRunning(UUID stepId) {
        if (stepId == null) {
            return;
        }
        steps.findById(stepId)
                .filter(step -> step.getState() == TaskStepState.OPEN)
                .ifPresent(step -> {
                    step.markState(TaskStepState.RUNNING);
                    steps.saveAndFlush(step);
                    publish(step.getTask().getId());
                });
    }

    /** Drop a step {@code RUNNING -> OPEN} when its session ends without claiming it. A no-op otherwise. */
    @Transactional
    public void releaseRunning(UUID stepId) {
        if (stepId == null) {
            return;
        }
        steps.findById(stepId)
                .filter(step -> step.getState() == TaskStepState.RUNNING)
                .ifPresent(step -> {
                    step.markState(TaskStepState.OPEN);
                    steps.saveAndFlush(step);
                    publish(step.getTask().getId());
                });
    }

    @Transactional
    public List<TaskStepView> move(UUID id, int position) {
        TaskStep step = require(id);
        UUID taskId = step.getTask().getId();

        List<TaskStep> ordered = new ArrayList<>(steps.findByTaskIdOrderByPositionAsc(taskId));
        ordered.removeIf(candidate -> candidate.getId().equals(id));
        ordered.add(Math.clamp(position, 0, ordered.size()), step);

        List<TaskStepView> settled = settleDraftsAtTail(ordered);
        steps.flush();
        publish(taskId);
        return settled;
    }

    /** Keep every {@code DRAFT} step after every non-draft one, order within each group preserved. */
    private void settleDraftsAtTail(UUID taskId) {
        settleDraftsAtTail(new ArrayList<>(steps.findByTaskIdOrderByPositionAsc(taskId)));
        steps.flush();
    }

    private List<TaskStepView> settleDraftsAtTail(List<TaskStep> ordered) {
        List<TaskStep> settled = new ArrayList<>(ordered.size());
        ordered.stream().filter(step -> !step.getState().draft()).forEach(settled::add);
        ordered.stream().filter(step -> step.getState().draft()).forEach(settled::add);
        renumber(settled);
        return settled.stream().map(TaskStepView::of).toList();
    }

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

    private void publish(UUID taskId) {
        List<TaskStepView> current = steps.findByTaskIdOrderByPositionAsc(taskId).stream()
                .map(TaskStepView::of).toList();
        events.publishEvent(new StepStreamEvent(taskId, current));
    }

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
