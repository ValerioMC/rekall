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
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button"));
        when(commitReferences.findByTaskIdAndCommitHash(taskId, "abc123")).thenReturn(List.of());

        CommitReferenceView logged = service.recordLatestCommit(taskId, null);

        assertThat(logged.taskId()).isEqualTo(taskId);
        assertThat(logged.stepId()).isNull();
        assertThat(logged.commitHash()).isEqualTo("abc123");
        assertThat(logged.comment()).isEqualTo("Wire up the button");
    }

    @Test
    @DisplayName("pressing the button twice on the same commit does not create a second row")
    void sameCommitIsNotLoggedTwice() {
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button"));
        CommitReference existing = new CommitReference(task, null, "abc123", "Wire up the button");
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

        CommitReference taskLevel = new CommitReference(task, null, "abc123", "Wire up the button");
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button"));
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
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", longSubject));
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
        when(git.head(any())).thenReturn(new GitHeadReader.Commit("abc123", "Wire up the button"));
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
}
