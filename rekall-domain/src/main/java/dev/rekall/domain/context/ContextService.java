package dev.rekall.domain.context;

import dev.rekall.domain.Company;
import dev.rekall.domain.Document;
import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.repository.CompanyRepository;
import dev.rekall.domain.repository.ProjectRepository;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.wrapup.WrapupView;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

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
    public static final List<String> ENTITY_NAMES = List.of("company", "project", "task");

    private final CompanyRepository companies;
    private final ProjectRepository projects;
    private final TaskRepository tasks;

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
            case "company", "companies", "azienda", "aziende", "cliente" -> companies
                    .findByNameIgnoreCase(value)
                    .map(this::render)
                    .orElseThrow(() -> new UnknownAnchorException("No company matches '%s'".formatted(value)));
            case "project", "projects", "progetto", "progetti" ->
                    renderSingleProject(projects.findByLabelIgnoreCase(value), value);
            case "task", "tasks", "issue", "issues", "attivita" -> renderSingleTask(tasks.findByLabelIgnoreCase(value), value);
            default -> throw new UnknownAnchorException(
                    "No entity matches '%s'. Known entities: %s".formatted(entityName, String.join(", ", ENTITY_NAMES)));
        };
    }

    /** The qualified two-part form, {@code project:vega task:report-builder}. */
    @Transactional(readOnly = true)
    public ContextRecord loadTask(String projectLabel, String taskLabel) {
        return tasks.findByProjectLabelIgnoreCaseAndLabelIgnoreCase(projectLabel, taskLabel)
                .map(this::render)
                .orElseThrow(() -> new UnknownAnchorException(
                        "No task '%s' on project '%s'".formatted(taskLabel, projectLabel)));
    }

    /** The positional form: every entity is tried and exactly one match is required. */
    private ContextRecord loadAnywhere(String value) {
        List<ContextRecord> matches = new ArrayList<>();
        companies.findByNameIgnoreCase(value).map(this::render).ifPresent(matches::add);
        projects.findByLabelIgnoreCase(value).stream().map(this::render).forEach(matches::add);
        tasks.findByLabelIgnoreCase(value).stream().map(this::render).forEach(matches::add);

        if (matches.isEmpty()) {
            throw new UnknownAnchorException(
                    "Nothing matches '%s'. Known entities: %s".formatted(value, String.join(", ", ENTITY_NAMES)));
        }
        if (matches.size() > 1) {
            throw new AmbiguousAnchorException(value, matches.stream().map(ContextRecord::anchor).toList());
        }
        return matches.getFirst();
    }

    /** The qualified two-part form for a project, {@code company:acme project:website}. */
    @Transactional(readOnly = true)
    public ContextRecord loadProject(String companyName, String projectLabel) {
        return projects.findByCompanyNameIgnoreCaseAndLabelIgnoreCase(companyName, projectLabel)
                .map(this::render)
                .orElseThrow(() -> new UnknownAnchorException(
                        "No project '%s' on company '%s'".formatted(projectLabel, companyName)));
    }

    private ContextRecord renderSingleProject(List<Project> found, String value) {
        if (found.isEmpty()) {
            throw new UnknownAnchorException("No project matches '%s'".formatted(value));
        }
        if (found.size() > 1) {
            throw new AmbiguousAnchorException(
                    value, found.stream().map(project -> "company:" + project.getCompany().getName()).toList());
        }
        return render(found.getFirst());
    }

    private ContextRecord renderSingleTask(List<Task> found, String value) {
        if (found.isEmpty()) {
            throw new UnknownAnchorException("No task matches '%s'".formatted(value));
        }
        if (found.size() > 1) {
            throw new AmbiguousAnchorException(
                    value, found.stream().map(task -> "project:" + task.getProject().getLabel()).toList());
        }
        return render(found.getFirst());
    }

    // ------------------------------------------------------------------ assembly

    private ContextRecord render(Company company) {
        Map<String, String> fields = new LinkedHashMap<>();
        putIfPresent(fields, "description", company.getDescription());

        return new ContextRecord(
                "Company",
                company.getName(),
                "company:" + company.getName(),
                fields,
                List.of(),
                company.getProjects().stream().map(project -> "project:" + project.getLabel()).toList(),
                List.of(),
                null,
                null);
    }

    /**
     * The heading carries the title and the anchor line carries the label.
     *
     * <p>Both are worth the tokens they cost. The title is what the record is called in the
     * conversation that follows; the label is what has to be typed to load it again, and a model
     * that has only seen the title will invent an anchor out of it.
     */
    private ContextRecord render(Project project) {
        Map<String, String> fields = new LinkedHashMap<>();
        fields.put("status", project.getStatus().name());
        putIfPresent(fields, "description", project.getDescription());

        return new ContextRecord(
                "Project",
                project.getTitle(),
                "project:" + project.getLabel(),
                fields,
                List.of(renderReferenced(project.getCompany())),
                project.getTasks().stream().map(task -> "task:" + task.getLabel()).toList(),
                List.of(),
                null,
                project.getBlueprintMarkdown());
    }

    /**
     * A task carries its project in full, every note attached to it, and its wrapup.
     *
     * <p>A note reached this way may well be attached to other tasks too. That is the point of
     * the relation: cluster access is written once and arrives with each task that needs it.
     *
     * <p>The wrapup is what closes the loop the whole feature exists for. A session ends by
     * writing what the implementation now looks like, and the next one opens on it, so the work
     * starts from the current state rather than from reading the code back.
     */
    private ContextRecord render(Task task) {
        Map<String, String> fields = new LinkedHashMap<>();
        fields.put("status", task.getStatus().name());
        putIfPresent(fields, "description", task.getDescription());

        List<ContextRecord> references = new ArrayList<>();
        references.add(renderReferenced(task.getProject()));

        return new ContextRecord(
                "Task",
                task.getTitle(),
                "task:" + task.getLabel(),
                fields,
                references,
                List.of(),
                documentsOf(task.getDocuments()),
                task.getWrapup() == null ? null : WrapupView.of(task.getWrapup()),
                null);
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
                full.kind(), full.label(), full.anchor(), full.fields(),
                full.references(), List.of(), full.documents(), null, full.blueprint());
    }

    private ContextRecord renderReferenced(Company company) {
        ContextRecord full = render(company);
        return new ContextRecord(
                full.kind(), full.label(), full.anchor(), full.fields(),
                List.of(), List.of(), full.documents(), null, full.blueprint());
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
