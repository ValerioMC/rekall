package dev.rekall.api.dto;

import dev.rekall.domain.Document;
import dev.rekall.domain.Environment;
import dev.rekall.domain.Project;
import dev.rekall.domain.ProjectStatus;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStatus;
import jakarta.validation.constraints.NotBlank;

import java.time.Instant;
import java.util.UUID;

/**
 * What crosses the HTTP boundary.
 *
 * <p>Mapped from entities inside the controller's transaction and never after it: a response
 * carrying a lazy association fails at serialisation time and nowhere else, which is the bug
 * the endpoint coverage test exists to catch.
 */
public final class ApiDtos {

    private ApiDtos() {
    }

    public record ProjectResponse(
            UUID id, String name, ProjectStatus status, String description, int taskCount, Instant updatedAt) {

        public static ProjectResponse of(Project project) {
            return new ProjectResponse(
                    project.getId(),
                    project.getName(),
                    project.getStatus(),
                    project.getDescription(),
                    project.getTasks().size(),
                    project.getUpdatedAt());
        }
    }

    public record ProjectRequest(@NotBlank String name, ProjectStatus status, String description) {
    }

    public record TaskResponse(
            UUID id,
            String name,
            TaskStatus status,
            String description,
            UUID projectId,
            String projectName,
            UUID environmentId,
            String environmentLabel,
            Instant updatedAt) {

        public static TaskResponse of(Task task) {
            Environment environment = task.getEnvironment();
            return new TaskResponse(
                    task.getId(),
                    task.getName(),
                    task.getStatus(),
                    task.getDescription(),
                    task.getProject().getId(),
                    task.getProject().getName(),
                    environment == null ? null : environment.getId(),
                    environment == null ? null : environment.getLabel(),
                    task.getUpdatedAt());
        }
    }

    public record TaskRequest(
            @NotBlank String name, TaskStatus status, String description, UUID projectId, UUID environmentId) {
    }

    public record EnvironmentResponse(
            UUID id, String label, String namespace, String kubeconfigPath, Instant updatedAt) {

        public static EnvironmentResponse of(Environment environment) {
            return new EnvironmentResponse(
                    environment.getId(),
                    environment.getLabel(),
                    environment.getNamespace(),
                    environment.getKubeconfigPath(),
                    environment.getUpdatedAt());
        }
    }

    public record EnvironmentRequest(@NotBlank String label, String namespace, String kubeconfigPath) {
    }

    public record DocumentResponse(
            UUID id, String title, String kind, String bodyMarkdown, String owner, int position, Instant updatedAt) {

        public static DocumentResponse of(Document document) {
            return new DocumentResponse(
                    document.getId(),
                    document.getTitle(),
                    document.getKind(),
                    document.getBodyMarkdown(),
                    ownerAnchor(document),
                    document.getPosition(),
                    document.getUpdatedAt());
        }

        private static String ownerAnchor(Document document) {
            if (document.getProject() != null) {
                return "project:" + document.getProject().getName();
            }
            if (document.getTask() != null) {
                return "task:" + document.getTask().getName();
            }
            return "environment:" + document.getEnvironment().getLabel();
        }
    }

    /** Exactly one owner id is set, matching the check constraint on the table. */
    public record DocumentRequest(
            @NotBlank String title,
            @NotBlank String kind,
            String bodyMarkdown,
            UUID projectId,
            UUID taskId,
            UUID environmentId) {
    }
}
