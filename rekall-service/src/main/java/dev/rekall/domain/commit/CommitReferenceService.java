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
 * Logs a commit of a task's project folder against that task, or one of its steps: the tip by
 * default, or any commit named by its hash. Both the console and {@code rekall_record_commit}
 * land here: same lookup, same git read, same row. A hash already logged for that (task, step)
 * pair is returned as-is rather than duplicated.
 */
@Service
@RequiredArgsConstructor
public class CommitReferenceService {

    private final TaskRepository tasks;
    private final TaskStepRepository steps;
    private final CommitReferenceRepository commitReferences;
    private final GitLogReader git;

    /** How far back the picker looks. Enough to find last week's commit, small enough to read at a glance. */
    public static final int RECENT_LIMIT = 30;

    /** Every commit logged so far, newest first: what the console shows next to each task and step. */
    @Transactional(readOnly = true)
    public List<CommitReferenceView> findAll() {
        return commitReferences.findAllByOrderByCreatedAtDesc().stream()
                .map(CommitReferenceView::of)
                .toList();
    }

    /** The recent log of the task's project folder, newest first: what the console offers when picking a commit by hand. */
    @Transactional(readOnly = true)
    public List<RecentCommitView> recentCommits(UUID taskId) {
        Task task = tasks.findById(taskId).orElseThrow(() -> new NotFoundException("task", taskId));
        return git.recent(repoFolderOf(task), RECENT_LIMIT).stream().map(RecentCommitView::of).toList();
    }

    /** Called from the console: the caller already knows {@code taskId}/{@code stepId} from the terminal it opened. */
    @Transactional
    public CommitReferenceView recordLatestCommit(UUID taskId, UUID stepId) {
        return recordCommit(taskId, stepId, null);
    }

    /**
     * Called from the console's picker: a commit chosen from the recent log or pasted by hand.
     * A null or blank {@code commitHash} means the tip, same as {@link #recordLatestCommit}.
     */
    @Transactional
    public CommitReferenceView recordCommit(UUID taskId, UUID stepId, String commitHash) {
        Task task = tasks.findById(taskId).orElseThrow(() -> new NotFoundException("task", taskId));
        return record(task, resolveStepById(task, stepId), commitHash);
    }

    /** Called from the MCP tool: anchored the way every other tool anchors a task, plus an optional step reference. */
    @Transactional
    public CommitReferenceView recordLatestCommit(String projectLabel, String taskLabel, String stepRef) {
        return recordCommit(projectLabel, taskLabel, stepRef, null);
    }

    /** The MCP path with a hash: a session logging a commit it made earlier rather than the one it just made. */
    @Transactional
    public CommitReferenceView recordCommit(String projectLabel, String taskLabel, String stepRef, String commitHash) {
        Task task = resolveTask(projectLabel, taskLabel);
        TaskStep step = stepRef == null || stepRef.isBlank() ? null : resolveStepByRef(task, stepRef);
        return record(task, step, commitHash);
    }

    private CommitReferenceView record(Task task, TaskStep step, String commitHash) {
        Path folder = repoFolderOf(task);
        GitLogReader.Commit commit = commitHash == null || commitHash.isBlank()
                ? git.head(folder)
                : git.commit(folder, commitHash.strip());
        UUID stepId = step == null ? null : step.getId();

        Optional<CommitReference> existing = commitReferences.findByTaskIdAndCommitHash(task.getId(), commit.hash())
                .stream()
                .filter(reference -> Objects.equals(idOf(reference.getStep()), stepId))
                .findFirst();
        if (existing.isPresent()) {
            return CommitReferenceView.of(existing.get());
        }

        CommitReference reference = new CommitReference(
                task, step, commit.hash(), truncated(commit.subject()), truncatedDiff(commit.diff()));
        return CommitReferenceView.of(commitReferences.saveAndFlush(reference));
    }

    private static Path repoFolderOf(Task task) {
        String folder = task.getProject().getRepoFolder();
        if (folder == null || folder.isBlank()) {
            throw new IllegalArgumentException(
                    "Set this project's folder on its page before logging a commit here.");
        }
        return Path.of(folder.strip());
    }

    /** The diff stored for one logged commit, read on demand: the list endpoint stays light. */
    @Transactional(readOnly = true)
    public String diffFor(UUID id) {
        CommitReference reference =
                commitReferences.findById(id).orElseThrow(() -> new NotFoundException("commit reference", id));
        return reference.getDiff();
    }

    /**
     * Whether a logged commit rides along with {@code rekall_context}. A null flag reads as off, so a
     * body that omits it never turns a commit on by accident. Console-only, one row at a time.
     */
    @Transactional
    public CommitReferenceView setInContext(UUID id, Boolean inContext) {
        CommitReference reference =
                commitReferences.findById(id).orElseThrow(() -> new NotFoundException("commit reference", id));
        reference.setInContext(Boolean.TRUE.equals(inContext));
        return CommitReferenceView.of(commitReferences.saveAndFlush(reference));
    }

    /** Removes the link between a commit and the task/step it was logged against. Manual, from the console. */
    @Transactional
    public void delete(UUID id) {
        if (!commitReferences.existsById(id)) {
            throw new NotFoundException("commit reference", id);
        }
        commitReferences.deleteById(id);
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
