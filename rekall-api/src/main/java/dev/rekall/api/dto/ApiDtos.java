package dev.rekall.api.dto;

import dev.rekall.domain.Company;
import dev.rekall.domain.Document;
import dev.rekall.domain.Project;
import dev.rekall.domain.ProjectStatus;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStatus;
import dev.rekall.domain.TaskStep;
import dev.rekall.domain.TaskStepState;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

import java.time.Instant;
import java.util.List;
import java.util.UUID;

public final class ApiDtos {

    private ApiDtos() {
    }

    public record CompanyResponse(
            UUID id, String name, String description, int projectCount, int taskCount, Instant updatedAt) {

        public static CompanyResponse of(Company company) {
            return new CompanyResponse(
                    company.getId(),
                    company.getName(),
                    company.getDescription(),
                    company.getProjects().size(),
                    company.getProjects().stream().mapToInt(project -> project.getTasks().size()).sum(),
                    company.getUpdatedAt());
        }
    }

    public record CompanyRequest(@NotBlank String name, String description) {
    }

    public record ProjectResponse(
            UUID id,
            String label,
            String title,
            ProjectStatus status,
            String description,
            String blueprintMarkdown,
            String repoFolder,
            UUID companyId,
            String companyName,
            int taskCount,
            String anchor,
            Instant updatedAt) {

        public static ProjectResponse of(Project project) {
            return new ProjectResponse(
                    project.getId(),
                    project.getLabel(),
                    project.getTitle(),
                    project.getStatus(),
                    project.getDescription(),
                    project.getBlueprintMarkdown(),
                    project.getRepoFolder(),
                    project.getCompany().getId(),
                    project.getCompany().getName(),
                    project.getTasks().size(),
                    "project:" + project.getLabel(),
                    project.getUpdatedAt());
        }
    }

    public record ProjectRequest(
            @NotBlank String label,
            @NotBlank String title,
            ProjectStatus status,
            String description,
            String blueprintMarkdown,
            String repoFolder,
            UUID companyId) {
    }

    public record TaskResponse(
            UUID id,
            String label,
            String title,
            TaskStatus status,
            String description,
            UUID projectId,
            String projectLabel,
            String projectTitle,
            String companyName,
            String projectRepoFolder,
            int documentCount,
            int stepCount,
            int stepsDone,
            boolean hasWrapup,
            TaskStepState reviewState,
            boolean reviewActive,
            Instant claimedAt,
            Instant acceptedAt,
            String reviewNote,
            String anchor,
            Instant updatedAt) {

        public static TaskResponse of(Task task) {
            return new TaskResponse(
                    task.getId(),
                    task.getLabel(),
                    task.getTitle(),
                    task.getStatus(),
                    task.getDescription(),
                    task.getProject().getId(),
                    task.getProject().getLabel(),
                    task.getProject().getTitle(),
                    task.getProject().getCompany().getName(),
                    task.getProject().getRepoFolder(),
                    task.getDocuments().size(),
                    task.getSteps().size(),
                    (int) task.getSteps().stream().filter(TaskStep::isDone).count(),
                    task.getWrapup() != null,
                    task.getReviewState(),
                    task.reviewActive(),
                    task.getClaimedAt(),
                    task.getAcceptedAt(),
                    task.getReviewNote(),
                    "project:%s task:%s".formatted(task.getProject().getLabel(), task.getLabel()),
                    task.getUpdatedAt());
        }
    }

    public record TaskRequest(
            @NotBlank String label,
            @NotBlank String title,
            TaskStatus status,
            String description,
            UUID projectId) {
    }

    /**
     * The console's Accept ({@code reviewState = DONE}) and Send back
     * ({@code reviewState = OPEN}) for a stepless task. {@code RUNNING} follows a
     * live session and {@code CLAIMED} is the wrapup's to set, so neither is
     * accepted here.
     */
    public record TaskReviewRequest(@NotNull TaskStepState reviewState, String note) {
    }

    public record DocumentResponse(
            UUID id,
            String title,
            String kind,
            String bodyMarkdown,
            List<TaskRef> tasks,
            Instant updatedAt) {

        public static DocumentResponse of(Document document) {
            return new DocumentResponse(
                    document.getId(),
                    document.getTitle(),
                    document.getKind(),
                    document.getBodyMarkdown(),
                    document.getTasks().stream().map(TaskRef::of).toList(),
                    document.getUpdatedAt());
        }
    }

    public record TaskRef(
            UUID id,
            String label,
            String title,
            String projectLabel,
            String projectTitle,
            String companyName,
            String anchor) {

        public static TaskRef of(Task task) {
            return new TaskRef(
                    task.getId(),
                    task.getLabel(),
                    task.getTitle(),
                    task.getProject().getLabel(),
                    task.getProject().getTitle(),
                    task.getProject().getCompany().getName(),
                    "project:%s task:%s".formatted(task.getProject().getLabel(), task.getLabel()));
        }
    }

    public record DocumentRequest(
            @NotBlank String title,
            @NotBlank String kind,
            String bodyMarkdown,
            List<UUID> taskIds) {
    }

    public record WrapupRequest(@NotBlank String bodyMarkdown) {
    }

    public record TaskStepRequest(@NotBlank String title, String bodyMarkdown) {
    }

    public record TaskStepPatchRequest(String title, String bodyMarkdown, Boolean done) {
    }

    public record TaskStepMoveRequest(@NotNull Integer position) {
    }

    public record TimeEntryEditRequest(@NotNull Instant startedAt, Instant stoppedAt) {
    }
}
