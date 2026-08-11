package dev.rekall.api.service;

import dev.rekall.api.dto.ApiDtos.EnvironmentRequest;
import dev.rekall.api.dto.ApiDtos.EnvironmentResponse;
import dev.rekall.api.dto.ApiDtos.ProjectRequest;
import dev.rekall.api.dto.ApiDtos.ProjectResponse;
import dev.rekall.api.dto.ApiDtos.TaskRequest;
import dev.rekall.api.dto.ApiDtos.TaskResponse;
import dev.rekall.domain.Environment;
import dev.rekall.domain.Project;
import dev.rekall.domain.ProjectStatus;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStatus;
import dev.rekall.domain.repository.EnvironmentRepository;
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

    private final ProjectRepository projects;
    private final TaskRepository tasks;
    private final EnvironmentRepository environments;

    // ------------------------------------------------------------------ projects

    @Transactional(readOnly = true)
    public List<ProjectResponse> listProjects() {
        return projects.findAllByOrderByNameAsc().stream().map(ProjectResponse::of).toList();
    }

    @Transactional(readOnly = true)
    public ProjectResponse getProject(UUID id) {
        return ProjectResponse.of(requireProject(id));
    }

    @Transactional
    public ProjectResponse createProject(ProjectRequest request) {
        Project project = new Project(request.name());
        applyTo(project, request);
        return ProjectResponse.of(projects.save(project));
    }

    @Transactional
    public ProjectResponse updateProject(UUID id, ProjectRequest request) {
        Project project = requireProject(id);
        project.setName(request.name());
        applyTo(project, request);
        return ProjectResponse.of(project);
    }

    @Transactional
    public void deleteProject(UUID id) {
        projects.delete(requireProject(id));
    }

    private void applyTo(Project project, ProjectRequest request) {
        project.setDescription(request.description());
        project.setStatus(request.status() == null ? ProjectStatus.ACTIVE : request.status());
    }

    // ------------------------------------------------------------------ tasks

    @Transactional(readOnly = true)
    public List<TaskResponse> listTasks(UUID projectId) {
        List<Task> found = projectId == null ? tasks.findAll() : tasks.findByProjectIdOrderByNameAsc(projectId);
        return found.stream().map(TaskResponse::of).toList();
    }

    @Transactional(readOnly = true)
    public TaskResponse getTask(UUID id) {
        return TaskResponse.of(requireTask(id));
    }

    @Transactional
    public TaskResponse createTask(TaskRequest request) {
        Task task = new Task(request.name());
        applyTo(task, request);
        return TaskResponse.of(tasks.save(task));
    }

    @Transactional
    public TaskResponse updateTask(UUID id, TaskRequest request) {
        Task task = requireTask(id);
        task.setName(request.name());
        applyTo(task, request);
        return TaskResponse.of(task);
    }

    @Transactional
    public void deleteTask(UUID id) {
        tasks.delete(requireTask(id));
    }

    private void applyTo(Task task, TaskRequest request) {
        task.setDescription(request.description());
        task.setStatus(request.status() == null ? TaskStatus.TODO : request.status());
        task.setProject(requireProject(request.projectId()));
        task.setEnvironment(request.environmentId() == null ? null : requireEnvironment(request.environmentId()));
    }

    // ------------------------------------------------------------------ environments

    @Transactional(readOnly = true)
    public List<EnvironmentResponse> listEnvironments() {
        return environments.findAllByOrderByLabelAsc().stream().map(EnvironmentResponse::of).toList();
    }

    @Transactional(readOnly = true)
    public EnvironmentResponse getEnvironment(UUID id) {
        return EnvironmentResponse.of(requireEnvironment(id));
    }

    @Transactional
    public EnvironmentResponse createEnvironment(EnvironmentRequest request) {
        Environment environment = new Environment(request.label());
        applyTo(environment, request);
        return EnvironmentResponse.of(environments.save(environment));
    }

    @Transactional
    public EnvironmentResponse updateEnvironment(UUID id, EnvironmentRequest request) {
        Environment environment = requireEnvironment(id);
        environment.setLabel(request.label());
        applyTo(environment, request);
        return EnvironmentResponse.of(environment);
    }

    /**
     * @throws ConflictException if tasks still run on it. The foreign key is {@code RESTRICT},
     *     so this only turns a database refusal into a message the UI can show.
     */
    @Transactional
    public void deleteEnvironment(UUID id) {
        Environment environment = requireEnvironment(id);
        if (!environment.getTasks().isEmpty()) {
            throw new ConflictException("%d task(s) still run on '%s'"
                    .formatted(environment.getTasks().size(), environment.getLabel()));
        }
        environments.delete(environment);
    }

    private void applyTo(Environment environment, EnvironmentRequest request) {
        environment.setNamespace(request.namespace());
        environment.setKubeconfigPath(request.kubeconfigPath());
    }

    // ------------------------------------------------------------------ lookups

    Project requireProject(UUID id) {
        if (id == null) {
            throw new NotFoundException("A task must belong to a project");
        }
        return projects.findById(id).orElseThrow(() -> new NotFoundException("Project", id));
    }

    Task requireTask(UUID id) {
        return tasks.findById(id).orElseThrow(() -> new NotFoundException("Task", id));
    }

    Environment requireEnvironment(UUID id) {
        return environments.findById(id).orElseThrow(() -> new NotFoundException("Environment", id));
    }
}
