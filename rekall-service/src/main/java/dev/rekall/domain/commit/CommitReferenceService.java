package dev.rekall.domain.commit;

import dev.rekall.common.NotFoundException;
import dev.rekall.domain.CommitReference;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStep;
import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.CommitReferenceRepository;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TaskStepRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.nio.file.Path;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;

/**
 * Logs the tip commit of a task's project folder against that task, or one of its steps. Both the
 * button on a running terminal and {@code rekall_record_commit} land here: same lookup, same git
 * read, same row. A hash already logged for that (task, step) pair is returned as-is rather than
 * duplicated.
 */
@Service
@RequiredArgsConstructor
public class CommitReferenceService {

    private final TaskRepository tasks;
    private final TaskStepRepository steps;
    private final CommitReferenceRepository commitReferences;
    private final GitHeadReader git;

    /** Every commit logged so far, newest first: what the console shows next to each task and step. */
    @Transactional(readOnly = true)
    public List<CommitReferenceView> findAll() {
        return commitReferences.findAllByOrderByCreatedAtDesc().stream()
                .map(CommitReferenceView::of)
                .toList();
    }

    /** Called from the console: the caller already knows {@code taskId}/{@code stepId} from the terminal it opened. */
    @Transactional
    public CommitReferenceView recordLatestCommit(UUID taskId, UUID stepId) {
        Task task = tasks.findById(taskId).orElseThrow(() -> new NotFoundException("task", taskId));
        return record(task, resolveStepById(task, stepId));
    }

    /** Called from the MCP tool: anchored the way every other tool anchors a task, plus an optional step reference. */
    @Transactional
    public CommitReferenceView recordLatestCommit(String projectLabel, String taskLabel, String stepRef) {
        Task task = resolveTask(projectLabel, taskLabel);
        TaskStep step = stepRef == null || stepRef.isBlank() ? null : resolveStepByRef(task, stepRef);
        return record(task, step);
    }

    private CommitReferenceView record(Task task, TaskStep step) {
        String folder = task.getProject().getRepoFolder();
        if (folder == null || folder.isBlank()) {
            throw new IllegalArgumentException(
                    "Set this project's folder on its page before logging a commit here.");
        }

        GitHeadReader.Commit head = git.head(Path.of(folder.strip()));
        UUID stepId = step == null ? null : step.getId();

        Optional<CommitReference> existing = commitReferences.findByTaskIdAndCommitHash(task.getId(), head.hash())
                .stream()
                .filter(reference -> Objects.equals(idOf(reference.getStep()), stepId))
                .findFirst();
        if (existing.isPresent()) {
            return CommitReferenceView.of(existing.get());
        }

        CommitReference reference = new CommitReference(
                task, step, head.hash(), truncated(head.subject()), truncatedDiff(head.diff()));
        return CommitReferenceView.of(commitReferences.saveAndFlush(reference));
    }

    /** The diff stored for one logged commit, read on demand: the list endpoint stays light. */
    @Transactional(readOnly = true)
    public String diffFor(UUID id) {
        CommitReference reference =
                commitReferences.findById(id).orElseThrow(() -> new NotFoundException("commit reference", id));
        return reference.getDiff();
    }

    private TaskStep resolveStepById(Task task, UUID stepId) {
        if (stepId == null) {
            return null;
        }
        TaskStep step = steps.findById(stepId).orElseThrow(() -> new NotFoundException("step", stepId));
        if (!step.getTask().getId().equals(task.getId())) {
            throw new IllegalArgumentException("That step does not belong to this task.");
        }
        return step;
    }

    private TaskStep resolveStepByRef(Task task, String ref) {
        List<TaskStep> ordered = steps.findByTaskIdOrderByPositionAsc(task.getId());
        if (ordered.isEmpty()) {
            throw new IllegalArgumentException("That task has no steps to log this commit against.");
        }
        String trimmed = ref.strip();

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
        throw new IllegalArgumentException(
                "No step matches '%s'. Pass its number or its exact title.".formatted(ref));
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

    private static UUID idOf(TaskStep step) {
        return step == null ? null : step.getId();
    }

    private static String truncated(String subject) {
        String text = subject == null ? "" : subject.strip();
        if (text.isEmpty()) {
            return "(no commit message)";
        }
        return text.length() > CommitReference.COMMENT_MAX
                ? text.substring(0, CommitReference.COMMENT_MAX - 1) + "…"
                : text;
    }

    private static String truncatedDiff(String diff) {
        if (diff == null || diff.isBlank()) {
            return null;
        }
        return diff.length() > CommitReference.DIFF_MAX
                ? diff.substring(0, CommitReference.DIFF_MAX) + "\n…(diff truncated)"
                : diff;
    }
}
