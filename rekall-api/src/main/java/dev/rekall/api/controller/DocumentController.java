package dev.rekall.api.controller;

import dev.rekall.api.dto.ApiDtos.DocumentRequest;
import dev.rekall.api.dto.ApiDtos.DocumentResponse;
import dev.rekall.api.service.DocumentService;
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

@RestController
@RequestMapping("/api/documents")
@RequiredArgsConstructor
public class DocumentController {

    private final DocumentService documents;

    @GetMapping
    public List<DocumentResponse> list(
            @RequestParam(required = false) UUID projectId,
            @RequestParam(required = false) UUID taskId,
            @RequestParam(required = false) UUID environmentId) {
        return documents.listFor(projectId, taskId, environmentId);
    }

    @GetMapping("/search")
    public List<DocumentResponse> search(@RequestParam String query) {
        return documents.search(query);
    }

    @PostMapping
    @ResponseStatus(HttpStatus.CREATED)
    public DocumentResponse create(@Valid @RequestBody DocumentRequest request) {
        return documents.create(request);
    }

    @PutMapping("/{id}")
    public DocumentResponse update(@PathVariable UUID id, @Valid @RequestBody DocumentRequest request) {
        return documents.update(id, request);
    }

    @DeleteMapping("/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void delete(@PathVariable UUID id) {
        documents.delete(id);
    }
}
