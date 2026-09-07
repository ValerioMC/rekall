package dev.rekall.api.controller;

import dev.rekall.api.dto.ApiDtos.TaskStepMoveRequest;
import dev.rekall.api.dto.ApiDtos.TaskStepPatchRequest;
import dev.rekall.api.dto.ApiDtos.TaskStepRequest;
import dev.rekall.domain.step.TaskStepService;
import dev.rekall.domain.step.TaskStepView;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PatchMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.UUID;

@RestController
@RequestMapping("/api")
@RequiredArgsConstructor
public class TaskStepController {

    private final TaskStepService steps;

    @GetMapping("/steps")
    public List<TaskStepView> list() {
        return steps.findAll();
    }

    @GetMapping("/tasks/{taskId}/steps")
    public List<TaskStepView> listOn(@PathVariable UUID taskId) {
        return steps.findByTask(taskId);
    }

    @PostMapping("/tasks/{taskId}/steps")
    @ResponseStatus(HttpStatus.CREATED)
    public TaskStepView add(@PathVariable UUID taskId, @Valid @RequestBody TaskStepRequest request) {
        return steps.add(taskId, request.title(), request.bodyMarkdown());
    }

    @PatchMapping("/steps/{id}")
    public TaskStepView edit(@PathVariable UUID id, @Valid @RequestBody TaskStepPatchRequest request) {
        return steps.edit(id, request.title(), request.bodyMarkdown(), request.done());
    }

    @PostMapping("/steps/{id}/move")
    public List<TaskStepView> move(@PathVariable UUID id, @Valid @RequestBody TaskStepMoveRequest request) {
        return steps.move(id, request.position());
    }

    @DeleteMapping("/steps/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void delete(@PathVariable UUID id) {
        steps.delete(id);
    }
}
