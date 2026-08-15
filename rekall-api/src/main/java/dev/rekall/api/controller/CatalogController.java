package dev.rekall.api.controller;

import dev.rekall.api.dto.ApiDtos.ProjectRequest;
import dev.rekall.api.dto.ApiDtos.ProjectResponse;
import dev.rekall.api.dto.ApiDtos.TaskRequest;
import dev.rekall.api.dto.ApiDtos.TaskResponse;
import dev.rekall.api.service.CatalogService;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.UUID;

/** CRUD for the three entities, which is now the whole write surface of the application. */
@RestController
@RequestMapping("/api")
@RequiredArgsConstructor
public class CatalogController {

    private final CatalogService catalog;

    @GetMapping("/projects")
    public List<ProjectResponse> listProjects() {
        return catalog.listProjects();
    }

    @GetMapping("/projects/{id}")
    public ProjectResponse getProject(@PathVariable UUID id) {
        return catalog.getProject(id);
    }

    @PostMapping("/projects")
    @ResponseStatus(HttpStatus.CREATED)
    public ProjectResponse createProject(@Valid @RequestBody ProjectRequest request) {
        return catalog.createProject(request);
    }

    @PutMapping("/projects/{id}")
    public ProjectResponse updateProject(@PathVariable UUID id, @Valid @RequestBody ProjectRequest request) {
        return catalog.updateProject(id, request);
    }

    @DeleteMapping("/projects/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void deleteProject(@PathVariable UUID id) {
        catalog.deleteProject(id);
    }

    @GetMapping("/tasks")
    public List<TaskResponse> listTasks(@RequestParam(required = false) UUID projectId) {
        return catalog.listTasks(projectId);
    }

    @GetMapping("/tasks/{id}")
    public TaskResponse getTask(@PathVariable UUID id) {
        return catalog.getTask(id);
    }

    @PostMapping("/tasks")
    @ResponseStatus(HttpStatus.CREATED)
    public TaskResponse createTask(@Valid @RequestBody TaskRequest request) {
        return catalog.createTask(request);
    }

    @PutMapping("/tasks/{id}")
    public TaskResponse updateTask(@PathVariable UUID id, @Valid @RequestBody TaskRequest request) {
        return catalog.updateTask(id, request);
    }

    @DeleteMapping("/tasks/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void deleteTask(@PathVariable UUID id) {
        catalog.deleteTask(id);
    }

}
