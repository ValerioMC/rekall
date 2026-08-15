package dev.rekall.api.controller;

import dev.rekall.api.dto.ApiDtos.WrapupRequest;
import dev.rekall.api.service.NotFoundException;
import dev.rekall.domain.WrapupAuthor;
import dev.rekall.domain.wrapup.WrapupService;
import dev.rekall.domain.wrapup.WrapupView;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.UUID;

/**
 * The console's half of the wrapup.
 *
 * <p>Claude writes one over MCP at the end of a session; this is where it is corrected. Both
 * go through the same {@code WrapupService}, so there is one definition of what a wrapup may
 * be and one place that enforces it, and the only thing that differs is which
 * {@link WrapupAuthor} ends up on the row.
 *
 * <p>{@code PUT} rather than {@code POST}: a task has one wrapup at the address of that task,
 * and writing to it either creates or replaces. There is no collection to post into.
 */
@RestController
@RequestMapping("/api")
@RequiredArgsConstructor
public class WrapupController {

    private final WrapupService wrapups;

    /**
     * Every wrapup at once.
     *
     * <p>Read whole, the way the notes are: the console keeps the catalogue in memory so that
     * selecting a task shows its state without a round trip, and there are at most as many
     * wrapups as there are tasks.
     */
    @GetMapping("/wrapups")
    public List<WrapupView> list() {
        return wrapups.findAll();
    }

    @GetMapping("/tasks/{taskId}/wrapup")
    public WrapupView get(@PathVariable UUID taskId) {
        return wrapups.find(taskId)
                .orElseThrow(() -> new NotFoundException("This task has no wrapup yet"));
    }

    @PutMapping("/tasks/{taskId}/wrapup")
    public WrapupView write(@PathVariable UUID taskId, @Valid @RequestBody WrapupRequest request) {
        return wrapups.write(taskId, request.bodyMarkdown(), WrapupAuthor.HAND).wrapup();
    }

    /** Removing the wrapup, not the task. Silent when there was none: the end state is the same. */
    @DeleteMapping("/tasks/{taskId}/wrapup")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void delete(@PathVariable UUID taskId) {
        wrapups.delete(taskId);
    }
}
