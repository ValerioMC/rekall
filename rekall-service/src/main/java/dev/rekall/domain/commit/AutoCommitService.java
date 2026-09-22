package dev.rekall.domain.commit;

import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStep;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TaskStepRepository;
import lombok.RequiredArgsConstructor;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.nio.file.Path;
import java.util.List;
import java.util.UUID;

/**
 * Commits a project's repo folder when a session claims work on it, if the project asks for it.
 * A step claimed through {@code rekall_step} commits against that step; a Claude-authored wrapup
 * on a stepless task commits against the task, since that wrapup is what claims it. A task with a
 * checklist never commits on its wrapup: its steps carry the claims.
 *
 * <p>The commit is everything in the working tree, staged and committed as the identity git is
 * configured with in that folder, under a message {@link CommitMessageGenerator} writes: the
 * claiming session's own message when it sent one, otherwise one drawn from the step's detail or
 * the task's wrapup. It is then logged the way {@code rekall_record_commit} logs one. Nothing here
 * throws past the outcome: the claim that triggered it has already been written and stays written.
 */
@Service
@RequiredArgsConstructor
public class AutoCommitService {

    private static final Logger log = LoggerFactory.getLogger(AutoCommitService.class);

    private final TaskRepository tasks;
    private final TaskStepRepository steps;
    private final GitRepositoryInspector inspector;
    private final GitCommitter committer;
    private final CommitReferenceService commitReferences;

    @Transactional
    public AutoCommitOutcome afterStepClaim(UUID taskId, UUID stepId) {
        return afterStepClaim(taskId, stepId, null);
    }

    /**
     * @param sessionMessage the commit message the claiming session wrote, subject line first, or
     *                       {@code null} to have one derived from the step
     */
    @Transactional
    public AutoCommitOutcome afterStepClaim(UUID taskId, UUID stepId, String sessionMessage) {
        Task task = tasks.findById(taskId).orElse(null);
        TaskStep step = steps.findById(stepId).orElse(null);
        if (task == null || step == null) {
            return AutoCommitOutcome.notApplicable();
        }
        return commit(task, step, sessionMessage);
    }

    @Transactional
    public AutoCommitOutcome afterWrapup(UUID taskId) {
        return afterWrapup(taskId, null);
    }

    /**
     * @param sessionMessage the commit message the session wrote with the wrapup, or {@code null}
     *                       to have one derived from the wrapup itself
     */
    @Transactional
    public AutoCommitOutcome afterWrapup(UUID taskId, String sessionMessage) {
        Task task = tasks.findById(taskId).orElse(null);
        if (task == null || !task.reviewActive()) {
            return AutoCommitOutcome.notApplicable();
        }
        return commit(task, null, sessionMessage);
    }

    private AutoCommitOutcome commit(Task task, TaskStep step, String sessionMessage) {
        Project project = task.getProject();
        if (!project.isAutoCommit()) {
            return AutoCommitOutcome.notApplicable();
        }
        RepositoryStatus status = inspector.inspect(project.getRepoFolder());
        if (!status.repository()) {
            return AutoCommitOutcome.failed(
                    "this project's folder (" + project.getRepoFolder() + ") is not a git repository.");
        }
        if (status.userEmail() == null) {
            return AutoCommitOutcome.failed(
                    "git has no user.email in " + status.folder() + "; set it with `git config --global user.email`.");
        }

        Path folder = Path.of(status.folder());
        try {
            List<PendingChange> changes = committer.pendingChanges(folder);
            if (changes.isEmpty()) {
                return AutoCommitOutcome.skipped("nothing to commit in " + folder + ", so nothing was logged.");
            }
            String message = CommitMessageGenerator.generate(subjectOf(task, step), changes, sessionMessage);
            String hash = committer.commitAll(folder, message);
            log.info("Auto-committed {} in {} for task {} step {}", hash, folder, task.getLabel(),
                    step == null ? "-" : step.getTitle());
            return AutoCommitOutcome.committed(
                    commitReferences.recordCommit(task.getId(), step == null ? null : step.getId(), hash));
        } catch (IllegalArgumentException refused) {
            log.warn("Auto-commit refused in {} for task {}: {}", folder, task.getLabel(), refused.getMessage());
            return AutoCommitOutcome.failed(refused.getMessage());
        }
    }

    private static CommitMessageGenerator.Subject subjectOf(Task task, TaskStep step) {
        String anchor = "project:%s task:%s".formatted(task.getProject().getLabel(), task.getLabel());
        if (step == null) {
            String wrapup = task.getWrapup() == null ? null : task.getWrapup().getBodyMarkdown();
            return new CommitMessageGenerator.Subject(task.getTitle(), anchor, null, wrapup);
        }
        return new CommitMessageGenerator.Subject(
                step.getTitle(), anchor, step.getPosition() + 1, step.getBodyMarkdown());
    }
}
