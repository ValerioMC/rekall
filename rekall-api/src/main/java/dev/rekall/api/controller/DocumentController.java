package dev.rekall.api.controller;

import dev.rekall.content.DocumentMatch;
import dev.rekall.content.DocumentService;
import dev.rekall.meta.domain.Document;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
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

import java.time.Instant;
import java.util.List;
import java.util.UUID;

@RestController
@RequestMapping("/api/documents")
public class DocumentController {

    private final DocumentService documents;

    public DocumentController(DocumentService documents) {
        this.documents = documents;
    }

    @GetMapping
    public List<DocumentResponse> forRecord(@RequestParam String entity, @RequestParam UUID recordId) {
        return documents.forRecord(entity, recordId).stream().map(DocumentResponse::from).toList();
    }

    @GetMapping("/{id}")
    public DocumentResponse get(@PathVariable UUID id) {
        return DocumentResponse.from(documents.get(id));
    }

    @GetMapping("/search")
    public List<DocumentMatch> search(
            @RequestParam String query,
            @RequestParam(required = false) List<String> entities,
            @RequestParam(defaultValue = "20") int limit) {
        return documents.search(query, entities, limit);
    }

    @PostMapping
    @ResponseStatus(HttpStatus.CREATED)
    public DocumentResponse create(@Valid @RequestBody CreateRequest request) {
        return DocumentResponse.from(documents.create(
                request.entityName(), request.recordId(), request.title(), request.kind(), request.bodyMarkdown()));
    }

    @PutMapping("/{id}")
    public DocumentResponse update(@PathVariable UUID id, @Valid @RequestBody UpdateRequest request) {
        return DocumentResponse.from(
                documents.update(id, request.title(), request.kind(), request.bodyMarkdown()));
    }

    @DeleteMapping("/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void delete(@PathVariable UUID id) {
        documents.delete(id);
    }

    public record CreateRequest(
            @NotBlank String entityName,
            @NotNull UUID recordId,
            @NotBlank String title,
            @NotBlank String kind,
            String bodyMarkdown) {
    }

    public record UpdateRequest(@NotBlank String title, @NotBlank String kind, String bodyMarkdown) {
    }

    public record DocumentResponse(
            UUID id,
            String entityName,
            UUID recordId,
            String title,
            String kind,
            String bodyMarkdown,
            String sourcePath,
            int position,
            Instant updatedAt) {

        static DocumentResponse from(Document document) {
            return new DocumentResponse(
                    document.getId(),
                    document.getEntityName(),
                    document.getRecordId(),
                    document.getTitle(),
                    document.getKind(),
                    document.getBodyMarkdown(),
                    document.getSourcePath(),
                    document.getPosition(),
                    document.getUpdatedAt());
        }
    }
}
