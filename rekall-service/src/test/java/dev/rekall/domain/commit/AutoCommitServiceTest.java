package dev.rekall.domain.commit;

import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStep;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TaskStepRepository;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.mockito.ArgumentCaptor;

import java.nio.file.Path;
import java.time.Instant;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

class AutoCommitServiceTest {

    private static final String FOLDER = "/repo/vega";
    private static final RepositoryStatus READY =
            new RepositoryStatus(FOLDER, true, true, "main", "Test", "test@example.com");

    private final TaskRepository tasks = mock(TaskRepository.class);
    private final TaskStepRepository steps = mock(TaskStepRepository.class);
    private final GitRepositoryInspector inspector = mock(GitRepositoryInspector.class);
    private final GitCommitter committer = mock(GitCommitter.class);
    private final CommitReferenceService commitReferences = mock(CommitReferenceService.class);
    private final AutoCommitService service =
            new AutoCommitService(tasks, steps, inspector, committer, commitReferences);

    private final UUID taskId = UUID.randomUUID();
    private final UUID stepId = UUID.randomUUID();
    private Project project;
    private Task task;
    private TaskStep step;

    @BeforeEach
    void setUp() {
        project = mock(Project.class);
        when(project.getLabel()).thenReturn("vega");
        when(project.getRepoFolder()).thenReturn(FOLDER);
        when(project.isAutoCommit()).thenReturn(true);

        task = mock(Task.class);
        when(task.getId()).thenReturn(taskId);
        when(task.getLabel()).thenReturn("report-builder");
        when(task.getTitle()).thenReturn("Report builder");
        when(task.getProject()).thenReturn(project);
        when(task.reviewActive()).thenReturn(true);
        when(tasks.findById(taskId)).thenReturn(Optional.of(task));

        step = mock(TaskStep.class);
        when(step.getId()).thenReturn(stepId);
        when(step.getTitle()).thenReturn("Wire the export endpoint");
        when(step.getPosition()).thenReturn(2);
        when(steps.findById(stepId)).thenReturn(Optional.of(step));

        when(inspector.inspect(FOLDER)).thenReturn(READY);
    }

    @Test
    @DisplayName("a project that does not auto-commit does nothing, and never touches git")
    void aProjectThatDoesNotAutoCommitDoesNothing() {
        when(project.isAutoCommit()).thenReturn(false);

        AutoCommitOutcome outcome = service.afterStepClaim(taskId, stepId);

        assertThat(outcome.status()).isEqualTo(AutoCommitOutcome.Status.OFF);
        assertThat(outcome.describe()).isEmpty();
        verifyNoInteractions(inspector, committer, commitReferences);
    }

    @Test
    @DisplayName("a step claim commits the pending changes and logs the commit against that step")
    void aStepClaimCommitsAndLogsAgainstTheStep() {
        when(committer.pendingChanges(Path.of(FOLDER)))
                .thenReturn(List.of(new PendingChange(PendingChange.Kind.ADDED, "src/export.ts")));
        when(committer.commitAll(eq(Path.of(FOLDER)), anyString())).thenReturn("abc123");
        CommitReferenceView logged = view(stepId, "Wire the export endpoint", "abc123", "feat: Wire the export endpoint");
        when(commitReferences.recordCommit(taskId, stepId, "abc123")).thenReturn(logged);

        AutoCommitOutcome outcome = service.afterStepClaim(taskId, stepId);

        assertThat(outcome.status()).isEqualTo(AutoCommitOutcome.Status.COMMITTED);
        assertThat(outcome.reference()).isEqualTo(logged);
        ArgumentCaptor<String> message = ArgumentCaptor.forClass(String.class);
        verify(committer).commitAll(eq(Path.of(FOLDER)), message.capture());
        assertThat(message.getValue())
                .startsWith("feat: Wire the export endpoint\n\nproject:vega task:report-builder, step 3\n");
        assertThat(outcome.describe()).contains("`abc123`").contains("\"Wire the export endpoint\"");
    }

    @Test
    @DisplayName("a wrapup on a stepless task commits against the task itself")
    void aWrapupOnAStepplessTaskCommitsAgainstTheTask() {
        when(committer.pendingChanges(any()))
                .thenReturn(List.of(new PendingChange(PendingChange.Kind.MODIFIED, "README.md")));
        when(committer.commitAll(any(), anyString())).thenReturn("d0c5");
        when(commitReferences.recordCommit(taskId, null, "d0c5"))
                .thenReturn(view(null, null, "d0c5", "docs: Report builder"));

        AutoCommitOutcome outcome = service.afterWrapup(taskId);

        assertThat(outcome.status()).isEqualTo(AutoCommitOutcome.Status.COMMITTED);
        verify(commitReferences).recordCommit(taskId, null, "d0c5");
        assertThat(outcome.describe()).contains("against the task");
    }

    @Test
    @DisplayName("a wrapup on a task with a checklist commits nothing: its steps carry the claims")
    void aWrapupOnATaskWithStepsCommitsNothing() {
        when(task.reviewActive()).thenReturn(false);

        assertThat(service.afterWrapup(taskId).status()).isEqualTo(AutoCommitOutcome.Status.OFF);
        verifyNoInteractions(committer, commitReferences);
    }

    @Test
    @DisplayName("a clean working tree is skipped, and nothing is logged")
    void aCleanTreeIsSkipped() {
        when(committer.pendingChanges(any())).thenReturn(List.of());

        AutoCommitOutcome outcome = service.afterStepClaim(taskId, stepId);

        assertThat(outcome.status()).isEqualTo(AutoCommitOutcome.Status.SKIPPED);
        assertThat(outcome.describe()).contains("nothing to commit");
        verify(committer, never()).commitAll(any(), anyString());
        verifyNoInteractions(commitReferences);
    }

    @Test
    @DisplayName("a folder that is no longer a repository fails the commit, not the claim")
    void aFolderThatIsNotARepositoryFails() {
        when(inspector.inspect(FOLDER)).thenReturn(RepositoryStatus.notARepository(FOLDER));

        AutoCommitOutcome outcome = service.afterStepClaim(taskId, stepId);

        assertThat(outcome.status()).isEqualTo(AutoCommitOutcome.Status.FAILED);
        assertThat(outcome.describe()).contains("not a git repository").contains("The claim stands");
        verifyNoInteractions(committer);
    }

    @Test
    @DisplayName("a repository with no user.email is refused before anything is staged")
    void noEmailIsRefusedBeforeStaging() {
        when(inspector.inspect(FOLDER)).thenReturn(new RepositoryStatus(FOLDER, true, true, "main", null, null));

        AutoCommitOutcome outcome = service.afterStepClaim(taskId, stepId);

        assertThat(outcome.status()).isEqualTo(AutoCommitOutcome.Status.FAILED);
        assertThat(outcome.detail()).contains("user.email");
        verifyNoInteractions(committer);
    }

    @Test
    @DisplayName("git refusing the commit is reported as a failure with git's reason")
    void gitRefusingIsReported() {
        when(committer.pendingChanges(any()))
                .thenReturn(List.of(new PendingChange(PendingChange.Kind.ADDED, "a.ts")));
        when(committer.commitAll(any(), anyString()))
                .thenThrow(new IllegalArgumentException("git commit failed in /repo/vega: hook rejected"));

        AutoCommitOutcome outcome = service.afterStepClaim(taskId, stepId);

        assertThat(outcome.status()).isEqualTo(AutoCommitOutcome.Status.FAILED);
        assertThat(outcome.detail()).contains("hook rejected");
        verifyNoInteractions(commitReferences);
    }

    @Test
    @DisplayName("an unknown task or step is nothing to commit for")
    void anUnknownTaskOrStepIsNothingToCommitFor() {
        assertThat(service.afterStepClaim(UUID.randomUUID(), stepId).status()).isEqualTo(AutoCommitOutcome.Status.OFF);
        assertThat(service.afterStepClaim(taskId, UUID.randomUUID()).status()).isEqualTo(AutoCommitOutcome.Status.OFF);
        assertThat(service.afterWrapup(UUID.randomUUID()).status()).isEqualTo(AutoCommitOutcome.Status.OFF);
    }

    private CommitReferenceView view(UUID stepId, String stepTitle, String hash, String comment) {
        return new CommitReferenceView(UUID.randomUUID(), taskId, stepId, stepTitle, hash, comment, false, Instant.now());
    }
}
