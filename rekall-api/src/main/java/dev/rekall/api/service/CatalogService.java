package dev.rekall.api.service;

import dev.rekall.api.dto.ApiDtos.CompanyRequest;
import dev.rekall.api.dto.ApiDtos.CompanyResponse;
import dev.rekall.api.dto.ApiDtos.ProjectRequest;
import dev.rekall.api.dto.ApiDtos.ProjectResponse;
import dev.rekall.api.dto.ApiDtos.TaskRequest;
import dev.rekall.api.dto.ApiDtos.TaskResponse;
import dev.rekall.domain.Company;
import dev.rekall.domain.Project;
import dev.rekall.domain.ProjectStatus;
import dev.rekall.domain.Slug;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStatus;
import dev.rekall.domain.repository.CompanyRepository;
import dev.rekall.domain.repository.DocumentRepository;
import dev.rekall.domain.repository.ProjectRepository;
import dev.rekall.domain.repository.TaskRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.UUID;

/** Write side of the three entities. Everything the UI can change goes through here. */
@Service
@RequiredArgsConstructor
public class CatalogService {

    private final CompanyRepository companies;
    private final ProjectRepository projects;
    private final TaskRepository tasks;
    private final DocumentRepository documents;

    // ------------------------------------------------------------------ companies

    @Transactional(readOnly = true)
    public List<CompanyResponse> listCompanies() {
        return companies.findAllByOrderByNameAsc().stream().map(CompanyResponse::of).toList();
    }

    @Transactional
    public CompanyResponse createCompany(CompanyRequest request) {
        Company company = new Company(request.name());
        company.setDescription(request.description());
        return CompanyResponse.of(companies.saveAndFlush(company));
    }

    @Transactional
    public CompanyResponse updateCompany(UUID id, CompanyRequest request) {
        Company company = requireCompany(id);
        company.setName(request.name());
        company.setDescription(request.description());
        return CompanyResponse.of(companies.saveAndFlush(company));
    }

    /** Cascades to the company's projects and their tasks, so the orphan sweep applies here too. */
    @Transactional
    public void deleteCompany(UUID id) {
        companies.delete(requireCompany(id));
        companies.flush();
        documents.deleteAll(documents.findOrphans());
    }

    // ------------------------------------------------------------------ projects

    @Transactional(readOnly = true)
    public List<ProjectResponse> listProjects() {
        return projects.findAllByOrderByCompanyNameAscLabelAsc().stream().map(ProjectResponse::of).toList();
    }

    @Transactional(readOnly = true)
    public ProjectResponse getProject(UUID id) {
        return ProjectResponse.of(requireProject(id));
    }

    /**
     * Flushed before mapping, because {@code @CreationTimestamp} and {@code @UpdateTimestamp}
     * are written by Hibernate at flush time. Mapping the entity before that returns a response
     * whose {@code updatedAt} is still null, which the client is entitled to reject.
     */
    @Transactional
    public ProjectResponse createProject(ProjectRequest request) {
        Project project = new Project(Slug.of(request.label()), request.title().trim());
        applyTo(project, request);
        return ProjectResponse.of(projects.saveAndFlush(project));
    }

    /**
     * The label is editable like anything else, and changing it changes the anchor.
     *
     * <p>Nothing stored points at a label, so there is no reference to repair; what breaks is
     * what is written down outside the application, which is the interface's job to warn about.
     */
    @Transactional
    public ProjectResponse updateProject(UUID id, ProjectRequest request) {
        Project project = requireProject(id);
        project.setLabel(Slug.of(request.label()));
        project.setTitle(request.title().trim());
        applyTo(project, request);
        return ProjectResponse.of(projects.saveAndFlush(project));
    }

    /** Cascades to the project's tasks, so the same orphan sweep applies. */
    @Transactional
    public void deleteProject(UUID id) {
        projects.delete(requireProject(id));
        projects.flush();
        documents.deleteAll(documents.findOrphans());
    }

    private void applyTo(Project project, ProjectRequest request) {
        project.setDescription(request.description());
        project.setBlueprintMarkdown(request.blueprintMarkdown());
        project.setStatus(request.status() == null ? ProjectStatus.ACTIVE : request.status());
        project.setCompany(requireCompany(request.companyId()));
    }

    // ------------------------------------------------------------------ tasks

    @Transactional(readOnly = true)
    public List<TaskResponse> listTasks(UUID projectId) {
        List<Task> found = projectId == null
                ? tasks.findAllByOrderByProjectLabelAscLabelAsc()
                : tasks.findByProjectIdOrderByLabelAsc(projectId);
        return found.stream().map(TaskResponse::of).toList();
    }

    @Transactional(readOnly = true)
    public TaskResponse getTask(UUID id) {
        return TaskResponse.of(requireTask(id));
    }

    @Transactional
    public TaskResponse createTask(TaskRequest request) {
        Task task = new Task(Slug.of(request.label()), request.title().trim());
        applyTo(task, request);
        return TaskResponse.of(tasks.saveAndFlush(task));
    }

    @Transactional
    public TaskResponse updateTask(UUID id, TaskRequest request) {
        Task task = requireTask(id);
        task.setLabel(Slug.of(request.label()));
        task.setTitle(request.title().trim());
        applyTo(task, request);
        return TaskResponse.of(tasks.saveAndFlush(task));
    }

    /**
     * Deleting a task unlinks its notes rather than deleting them, because a note may be on
     * other tasks and those tasks still need it. What is swept up afterwards is only the notes
     * that are now on nothing at all: the join table cannot express that rule, since a row
     * that no longer exists cannot be checked.
     */
    @Transactional
    public void deleteTask(UUID id) {
        Task task = requireTask(id);
        List.copyOf(task.getDocuments()).forEach(task::detach);
        tasks.delete(task);
        tasks.flush();
        documents.deleteAll(documents.findOrphans());
    }

    private void applyTo(Task task, TaskRequest request) {
        task.setDescription(request.description());
        task.setStatus(request.status() == null ? TaskStatus.TODO : request.status());
        task.setProject(requireProject(request.projectId()));
    }

    // ------------------------------------------------------------------ lookups

    Company requireCompany(UUID id) {
        if (id == null) {
            throw new NotFoundException("A project must belong to a company");
        }
        return companies.findById(id).orElseThrow(() -> new NotFoundException("Company", id));
    }

    Project requireProject(UUID id) {
        if (id == null) {
            throw new NotFoundException("A task must belong to a project");
        }
        return projects.findById(id).orElseThrow(() -> new NotFoundException("Project", id));
    }

    Task requireTask(UUID id) {
        return tasks.findById(id).orElseThrow(() -> new NotFoundException("Task", id));
    }

}
