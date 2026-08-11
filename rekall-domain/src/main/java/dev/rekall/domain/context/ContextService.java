package dev.rekall.domain.context;

import dev.rekall.domain.Document;
import dev.rekall.domain.Environment;
import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.repository.EnvironmentRepository;
import dev.rekall.domain.repository.ProjectRepository;
import dev.rekall.domain.repository.TaskRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * Loads the working context named by a list of anchors.
 *
 * <p>This is the whole of what Rekall does at read time. It used to be a diff against
 * {@code information_schema}, a registry of runtime-defined entities and a dynamic query
 * builder; with the shape fixed at compile time it is three lookups and a walk over
 * associations Hibernate already knows how to follow.
 *
 * <p>Everything is assembled inside one read-only transaction and returned materialised. That
 * is what lets the caller render it without an open session, and it is why the lazy
 * associations declared on the entities are safe here and nowhere else.
 */
@Service
@RequiredArgsConstructor
public class ContextService {

    /** Entity names an anchor may use, in the order a bare value is searched. */
    public static final List<String> ENTITY_NAMES = List.of("project", "task", "environment");

    private final ProjectRepository projects;
    private final TaskRepository tasks;
    private final EnvironmentRepository environments;

    /**
     * @param entityName the entity part of the anchor, or null for the positional form
     * @throws UnknownAnchorException if nothing matches
     * @throws AmbiguousAnchorException if more than one record matches
     */
    @Transactional(readOnly = true)
    public ContextRecord load(String entityName, String value) {
        if (entityName == null) {
            return loadAnywhere(value);
        }
        return switch (entityName.toLowerCase()) {
            case "project", "projects", "progetto", "progetti" -> projects
                    .findByNameIgnoreCase(value)
                    .map(this::render)
                    .orElseThrow(() -> new UnknownAnchorException("No project matches '%s'".formatted(value)));
            case "task", "tasks", "issue", "issues", "attivita" -> renderSingleTask(tasks.findByNameIgnoreCase(value), value);
            case "environment", "environments", "env", "ambiente", "ambienti" -> environments
                    .findByLabelIgnoreCase(value)
                    .map(this::render)
                    .orElseThrow(() -> new UnknownAnchorException("No environment matches '%s'".formatted(value)));
            default -> throw new UnknownAnchorException(
                    "No entity matches '%s'. Known entities: %s".formatted(entityName, String.join(", ", ENTITY_NAMES)));
        };
    }

    /** The qualified two-part form, {@code project:stvv task:code-validator}. */
    @Transactional(readOnly = true)
    public ContextRecord loadTask(String projectName, String taskName) {
        return tasks.findByProjectNameIgnoreCaseAndNameIgnoreCase(projectName, taskName)
                .map(this::render)
                .orElseThrow(() -> new UnknownAnchorException(
                        "No task '%s' on project '%s'".formatted(taskName, projectName)));
    }

    /** The positional form: every entity is tried and exactly one match is required. */
    private ContextRecord loadAnywhere(String value) {
        List<ContextRecord> matches = new ArrayList<>();
        projects.findByNameIgnoreCase(value).map(this::render).ifPresent(matches::add);
        tasks.findByNameIgnoreCase(value).stream().map(this::render).forEach(matches::add);
        environments.findByLabelIgnoreCase(value).map(this::render).ifPresent(matches::add);

        if (matches.isEmpty()) {
            throw new UnknownAnchorException(
                    "Nothing matches '%s'. Known entities: %s".formatted(value, String.join(", ", ENTITY_NAMES)));
        }
        if (matches.size() > 1) {
            throw new AmbiguousAnchorException(value, matches.stream().map(ContextRecord::anchor).toList());
        }
        return matches.getFirst();
    }

    private ContextRecord renderSingleTask(List<Task> found, String value) {
        if (found.isEmpty()) {
            throw new UnknownAnchorException("No task matches '%s'".formatted(value));
        }
        if (found.size() > 1) {
            throw new AmbiguousAnchorException(
                    value, found.stream().map(task -> "project:" + task.getProject().getName()).toList());
        }
        return render(found.getFirst());
    }

    // ------------------------------------------------------------------ assembly

    private ContextRecord render(Project project) {
        Map<String, String> fields = new LinkedHashMap<>();
        fields.put("status", project.getStatus().name());
        putIfPresent(fields, "description", project.getDescription());

        return new ContextRecord(
                "Project",
                project.getName(),
                "project:" + project.getName(),
                fields,
                List.of(),
                project.getTasks().stream().map(task -> "task:" + task.getName()).toList(),
                documentsOf(project.getDocuments()));
    }

    /**
     * A task carries its project and its environment in full, each with their own notes. The
     * environment's markdown is the point: it holds the cluster coordinates that used to live
     * in a separate file nobody remembered to open.
     */
    private ContextRecord render(Task task) {
        Map<String, String> fields = new LinkedHashMap<>();
        fields.put("status", task.getStatus().name());
        putIfPresent(fields, "description", task.getDescription());

        List<ContextRecord> references = new ArrayList<>();
        references.add(renderReferenced(task.getProject()));
        Optional.ofNullable(task.getEnvironment()).map(this::render).ifPresent(references::add);

        return new ContextRecord(
                "Task",
                task.getName(),
                "task:" + task.getName(),
                fields,
                references,
                List.of(),
                documentsOf(task.getDocuments()));
    }

    private ContextRecord render(Environment environment) {
        Map<String, String> fields = new LinkedHashMap<>();
        putIfPresent(fields, "namespace", environment.getNamespace());
        putIfPresent(fields, "kubeconfig_path", environment.getKubeconfigPath());

        return new ContextRecord(
                "Environment",
                environment.getLabel(),
                "environment:" + environment.getLabel(),
                fields,
                List.of(),
                environment.getTasks().stream().map(task -> "task:" + task.getName()).toList(),
                documentsOf(environment.getDocuments()));
    }

    /**
     * A project reached through a task, rather than anchored directly.
     *
     * <p>Its task list is deliberately dropped here: you asked for one task, and listing its
     * forty siblings underneath it is the fan-out the inverse rule exists to prevent.
     */
    private ContextRecord renderReferenced(Project project) {
        ContextRecord full = render(project);
        return new ContextRecord(
                full.kind(), full.label(), full.anchor(), full.fields(), List.of(), List.of(), full.documents());
    }

    private List<DocumentView> documentsOf(List<Document> documents) {
        return documents.stream()
                .map(document -> new DocumentView(document.getTitle(), document.getKind(), document.getBodyMarkdown()))
                .toList();
    }

    private void putIfPresent(Map<String, String> fields, String key, String value) {
        if (value != null && !value.isBlank()) {
            fields.put(key, value);
        }
    }
}
