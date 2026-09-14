package dev.rekall.domain.commit;

import dev.rekall.common.NotFoundException;
import dev.rekall.domain.CommitReference;
import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStep;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.CommitReferenceRepository;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TaskStepRepository;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.mockito.ArgumentCaptor;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

class CommitReferenceServiceTest {

    private final TaskRepository tasks = mock(TaskRepository.class);
    private final TaskStepRepository steps = mock(TaskStepRepository.class);
    private final CommitReferenceRepository commitReferences = mock(CommitReferenceRepository.class);
    private final GitHeadReader git = mock(GitHeadReader.class);
    private final CommitReferenceService service =
            new CommitReferenceService(tasks, steps, commitReferences, git);

    private final UUID taskId = UUID.randomUUID();
    private Task task;
    private Project project;

    @BeforeEach
    void setUp() {
        project = mock(Project.class);
        when(project.getRepoFolder()).thenReturn("/repo/vega");
        when(project.getLabel()).thenReturn("vega");

        task = mock(Task.class);
        when(task.getId()).thenReturn(taskId);
        when(task.getProject()).thenReturn(project);
        when(task.getLabel()).thenReturn("report-builder");
        when(tasks.findById(taskId)).thenReturn(Optional.of(task));

        when(commitReferences.saveAndFlush(any())).thenAnswer(invocation -> invocation.getArgument(0));
    }

    @Test
    @DisplayName("logs the tip commit against the task when no step is given")
    void logsAgainstTheTaskWhenNoStepIsGiven() {
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button", "diff --git a/x b/x"));
        when(commitReferences.findByTaskIdAndCommitHash(taskId, "abc123")).thenReturn(List.of());

        CommitReferenceView logged = service.recordLatestCommit(taskId, null);

        assertThat(logged.taskId()).isEqualTo(taskId);
        assertThat(logged.stepId()).isNull();
        assertThat(logged.commitHash()).isEqualTo("abc123");
        assertThat(logged.comment()).isEqualTo("Wire up the button");
    }

    @Test
    @DisplayName("the commit's diff is stored alongside the hash and comment")
    void storesTheDiffAlongsideTheCommit() {
        when(git.head(any()))
                .thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button", "diff --git a/x b/x\n+line"));
        when(commitReferences.findByTaskIdAndCommitHash(taskId, "abc123")).thenReturn(List.of());
        ArgumentCaptor<CommitReference> saved = ArgumentCaptor.forClass(CommitReference.class);

        service.recordLatestCommit(taskId, null);

        verify(commitReferences).saveAndFlush(saved.capture());
        assertThat(saved.getValue().getDiff()).isEqualTo("diff --git a/x b/x\n+line");
    }

    @Test
    @DisplayName("a commit git could not diff is stored with no diff, rather than failing the whole log")
    void storesNoDiffWhenGitCouldNotProduceOne() {
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button", null));
        when(commitReferences.findByTaskIdAndCommitHash(taskId, "abc123")).thenReturn(List.of());
        ArgumentCaptor<CommitReference> saved = ArgumentCaptor.forClass(CommitReference.class);

        CommitReferenceView logged = service.recordLatestCommit(taskId, null);

        assertThat(logged.commitHash()).isEqualTo("abc123");
        verify(commitReferences).saveAndFlush(saved.capture());
        assertThat(saved.getValue().getDiff()).isNull();
    }

    @Test
    @DisplayName("an overlong diff is truncated rather than stored whole")
    void truncatesAnOverlongDiff() {
        String longDiff = "+".repeat(CommitReference.DIFF_MAX + 500);
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button", longDiff));
        when(commitReferences.findByTaskIdAndCommitHash(taskId, "abc123")).thenReturn(List.of());
        ArgumentCaptor<CommitReference> saved = ArgumentCaptor.forClass(CommitReference.class);

        service.recordLatestCommit(taskId, null);

        verify(commitReferences).saveAndFlush(saved.capture());
        assertThat(saved.getValue().getDiff())
                .hasSize(CommitReference.DIFF_MAX + "\n…(diff truncated)".length())
                .endsWith("…(diff truncated)");
    }

    @Test
    @DisplayName("the diff of a logged commit is fetched by its own id")
    void fetchesTheDiffOfALoggedCommitById() {
        UUID id = UUID.randomUUID();
        CommitReference reference = new CommitReference(task, null, "abc123", "Wire up the button", "the diff");
        when(commitReferences.findById(id)).thenReturn(Optional.of(reference));

        assertThat(service.diffFor(id)).isEqualTo("the diff");
    }

    @Test
    @DisplayName("fetching the diff of an unknown commit reference is refused")
    void refusesTheDiffOfAnUnknownCommitReference() {
        UUID id = UUID.randomUUID();
        when(commitReferences.findById(id)).thenReturn(Optional.empty());

        assertThatThrownBy(() -> service.diffFor(id)).isInstanceOf(NotFoundException.class);
    }

    @Test
    @DisplayName("pressing the button twice on the same commit does not create a second row")
    void sameCommitIsNotLoggedTwice() {
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button", "diff --git a/x b/x"));
        CommitReference existing = new CommitReference(task, null, "abc123", "Wire up the button", "diff --git a/x b/x");
        when(commitReferences.findByTaskIdAndCommitHash(taskId, "abc123")).thenReturn(List.of(existing));

        CommitReferenceView logged = service.recordLatestCommit(taskId, null);

        assertThat(logged.commitHash()).isEqualTo("abc123");
        verify(commitReferences, never()).saveAndFlush(any());
    }

    @Test
    @DisplayName("the same commit already logged against the task is logged separately for a step")
    void sameCommitCanBeLoggedAgainstATaskAndAStepSeparately() {
        UUID stepId = UUID.randomUUID();
        TaskStep step = mock(TaskStep.class);
        when(step.getId()).thenReturn(stepId);
        when(step.getTask()).thenReturn(task);
        when(steps.findById(stepId)).thenReturn(Optional.of(step));

        CommitReference taskLevel = new CommitReference(task, null, "abc123", "Wire up the button", "diff --git a/x b/x");
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button", "diff --git a/x b/x"));
        when(commitReferences.findByTaskIdAndCommitHash(taskId, "abc123")).thenReturn(List.of(taskLevel));

        CommitReferenceView logged = service.recordLatestCommit(taskId, stepId);

        assertThat(logged.stepId()).isEqualTo(stepId);
        verify(commitReferences).saveAndFlush(any());
    }

    @Test
    @DisplayName("a project with no folder set refuses rather than guessing a working directory")
    void refusesWhenTheProjectHasNoFolder() {
        when(project.getRepoFolder()).thenReturn(null);

        assertThatThrownBy(() -> service.recordLatestCommit(taskId, null))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("folder");
    }

    @Test
    @DisplayName("a step from another task is refused")
    void refusesAStepFromAnotherTask() {
        UUID stepId = UUID.randomUUID();
        Task otherTask = mock(Task.class);
        when(otherTask.getId()).thenReturn(UUID.randomUUID());
        TaskStep step = mock(TaskStep.class);
        when(step.getTask()).thenReturn(otherTask);
        when(steps.findById(stepId)).thenReturn(Optional.of(step));

        assertThatThrownBy(() -> service.recordLatestCommit(taskId, stepId))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("does not belong");
    }

    @Test
    @DisplayName("the subject line is truncated to the comment's character cap")
    void truncatesAnOverlongSubjectLine() {
        String longSubject = "x".repeat(CommitReference.COMMENT_MAX + 50);
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", longSubject, "diff --git a/x b/x"));
        when(commitReferences.findByTaskIdAndCommitHash(taskId, "abc123")).thenReturn(List.of());

        CommitReferenceView logged = service.recordLatestCommit(taskId, null);

        assertThat(logged.comment()).hasSize(CommitReference.COMMENT_MAX).endsWith("…");
    }

    @Test
    @DisplayName("an unknown task id is refused rather than silently doing nothing")
    void refusesAnUnknownTaskId() {
        UUID unknown = UUID.randomUUID();
        when(tasks.findById(unknown)).thenReturn(Optional.empty());

        assertThatThrownBy(() -> service.recordLatestCommit(unknown, null))
                .isInstanceOf(NotFoundException.class);
    }

    @Test
    @DisplayName("the MCP path resolves the task by anchor the same way rekall_wrapup does")
    void resolvesByProjectAndTaskLabelForTheMcpPath() {
        when(tasks.findByProjectLabelIgnoreCaseAndLabelIgnoreCase("vega", "report-builder"))
                .thenReturn(Optional.of(task));
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button", "diff --git a/x b/x"));
        when(commitReferences.findByTaskIdAndCommitHash(taskId, "abc123")).thenReturn(List.of());

        CommitReferenceView logged = service.recordLatestCommit("vega", "report-builder", null);

        assertThat(logged.commitHash()).isEqualTo("abc123");
    }

    @Test
    @DisplayName("an unknown task label on the MCP path is refused")
    void refusesAnUnknownTaskLabelOnTheMcpPath() {
        when(tasks.findByProjectLabelIgnoreCaseAndLabelIgnoreCase("vega", "missing"))
                .thenReturn(Optional.empty());

        assertThatThrownBy(() -> service.recordLatestCommit("vega", "missing", null))
                .isInstanceOf(UnknownAnchorException.class);
    }

    @Test
    @DisplayName("deleting a logged commit removes its row")
    void deletingALoggedCommitRemovesItsRow() {
        UUID id = UUID.randomUUID();
        when(commitReferences.existsById(id)).thenReturn(true);

        service.delete(id);

        verify(commitReferences).deleteById(id);
    }

    @Test
    @DisplayName("deleting an unknown commit reference is refused rather than silently doing nothing")
    void refusesDeletingAnUnknownCommitReference() {
        UUID id = UUID.randomUUID();
        when(commitReferences.existsById(id)).thenReturn(false);

        assertThatThrownBy(() -> service.delete(id)).isInstanceOf(NotFoundException.class);
        verify(commitReferences, never()).deleteById(any());
    }
}
