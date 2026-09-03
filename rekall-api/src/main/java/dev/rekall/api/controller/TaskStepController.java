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

/**
 * The console's checklist, and the only way into {@code task_step}.
 *
 * <p>There is no MCP tool beside this on purpose. A step says whether a piece of work is done,
 * and the session that did the work is the last thing that should be allowed to answer that;
 * the person who reviewed it ticks the box.
 *
 * <p>A step is created under its task, because that is where it belongs and the position it
 * gets is decided by what is already there. Everything after that addresses the step by its own
 * id: it can be renamed, ticked, detailed, moved or removed, and none of those need to name the
 * task again.
 */
@RestController
@RequestMapping("/api")
@RequiredArgsConstructor
public class TaskStepController {

    private final TaskStepService steps;

    /**
     * Every step at once, the way the wrapups and the sessions are read.
     *
     * <p>The console keeps the catalogue in memory, so selecting a task shows its checklist
     * without a round trip, and there are only ever as many steps as there is work.
     */
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

    /** Ticking, renaming and detailing are the same request with a different field filled in. */
    @PatchMapping("/steps/{id}")
    public TaskStepView edit(@PathVariable UUID id, @Valid @RequestBody TaskStepPatchRequest request) {
        return steps.edit(id, request.title(), request.bodyMarkdown(), request.done());
    }

    /**
     * Moving one step, answering with the whole list.
     *
     * <p>A move renumbers everything it displaced, so returning the moved row alone would leave
     * the client holding positions that are no longer true.
     */
    @PostMapping("/steps/{id}/move")
    public List<TaskStepView> move(@PathVariable UUID id, @Valid @RequestBody TaskStepMoveRequest request) {
        return steps.move(id, request.position());
    }

    /** Removing one step. The rest of the checklist closes the gap it leaves. */
    @DeleteMapping("/steps/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void delete(@PathVariable UUID id) {
        steps.delete(id);
    }
}
