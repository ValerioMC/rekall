package dev.rekall.api.dto;

import dev.rekall.domain.Company;
import dev.rekall.domain.Document;
import dev.rekall.domain.Project;
import dev.rekall.domain.ProjectStatus;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStatus;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

import java.time.Instant;
import java.util.List;
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

    /**
     * The label and the title both travel, because the interface needs both at once: the title
     * is what a row reads as, and the label is what the anchor next to it has to say.
     */
    public record ProjectResponse(
            UUID id,
            String label,
            String title,
            ProjectStatus status,
            String description,
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
                    project.getCompany().getId(),
                    project.getCompany().getName(),
                    project.getTasks().size(),
                    "project:" + project.getLabel(),
                    project.getUpdatedAt());
        }
    }

    /**
     * The label is normalised to a slug on the way in, so what the client sent and what the
     * anchor will be are allowed to differ. The response carries the stored one.
     */
    public record ProjectRequest(
            @NotBlank String label,
            @NotBlank String title,
            ProjectStatus status,
            String description,
            UUID companyId) {
    }

    /**
     * {@code hasWrapup} rather than the wrapup itself, because this is what a row in a list
     * needs: whether this task has said what it currently is. The body is fetched from its own
     * endpoint by the one pane that shows it.
     */
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
            int documentCount,
            boolean hasWrapup,
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
                    task.getDocuments().size(),
                    task.getWrapup() != null,
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
     * A note reports every task it is on, because the interface has to show that editing this
     * one changes what three other tasks load. A single owner field would hide exactly that.
     */
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

    /** Enough of a task to name it and to link to it, without dragging the record along. */
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

    /**
     * The whole wrapup, every time.
     *
     * <p>One field, and no partial form: a wrapup is replaced rather than amended, so a request
     * that carried only the part that changed would be describing a diff, which is the one
     * thing a wrapup is defined against.
     *
     * <p>There is no response record beside this one. {@code WrapupView} is already materialised
     * for the MCP side and already carries the anchor, and a second record with the same ten
     * fields would be a boundary that exists only on paper.
     */
    public record WrapupRequest(@NotBlank String bodyMarkdown) {
    }

    /**
     * A hand correction to one session. {@code stoppedAt} is nullable and null only means
     * something when the session was already open; {@code TimeEntryService} is what decides
     * whether that is allowed.
     */
    public record TimeEntryEditRequest(@NotNull Instant startedAt, Instant stoppedAt) {
    }
}
