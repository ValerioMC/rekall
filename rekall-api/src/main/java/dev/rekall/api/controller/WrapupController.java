package dev.rekall.api.controller;

import dev.rekall.api.dto.ApiDtos.WrapupRequest;
import dev.rekall.common.NotFoundException;
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

@RestController
@RequestMapping("/api")
@RequiredArgsConstructor
public class WrapupController {

    private final WrapupService wrapups;

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

    @DeleteMapping("/tasks/{taskId}/wrapup")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void delete(@PathVariable UUID taskId) {
        wrapups.delete(taskId);
    }
}
