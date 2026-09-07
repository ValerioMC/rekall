package dev.rekall.api.service;

import dev.rekall.api.dto.ApiDtos.DocumentRequest;
import dev.rekall.api.dto.ApiDtos.DocumentResponse;
import dev.rekall.domain.Document;
import dev.rekall.domain.Task;
import dev.rekall.domain.repository.DocumentRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import java.util.UUID;

@Service
@RequiredArgsConstructor
public class DocumentService {

    private final DocumentRepository documents;
    private final CatalogService catalog;

    @Transactional(readOnly = true)
    public List<DocumentResponse> list(UUID taskId, UUID projectId) {
        if (taskId != null) {
            return map(documents.findByTasksIdOrderByTitleAsc(taskId));
        }
        if (projectId != null) {
            return map(documents.findByProject(projectId));
        }
        return map(documents.findAllByOrderByUpdatedAtDesc());
    }

    @Transactional(readOnly = true)
    public List<DocumentResponse> search(String term) {
        if (term == null || term.isBlank()) {
            return List.of();
        }
        return map(documents.search(term.trim()));
    }

    @Transactional
    public DocumentResponse create(DocumentRequest request) {
        Document document = new Document(request.title(), request.kind(), body(request));
        documents.save(document);
        link(document, resolve(request.taskIds()));
        documents.flush();
        return DocumentResponse.of(document);
    }

    @Transactional
    public DocumentResponse update(UUID id, DocumentRequest request) {
        Document document = require(id);
        document.setTitle(request.title());
        document.setKind(request.kind());
        document.setBodyMarkdown(body(request));
        link(document, resolve(request.taskIds()));
        documents.flush();
        return DocumentResponse.of(document);
    }

    @Transactional
    public void delete(UUID id) {
        Document document = require(id);
        Set.copyOf(document.getTasks()).forEach(task -> task.detach(document));
        documents.delete(document);
    }

    private void link(Document document, Set<Task> wanted) {
        Set.copyOf(document.getTasks()).stream()
                .filter(task -> !wanted.contains(task))
                .forEach(task -> task.detach(document));
        wanted.forEach(task -> task.attach(document));
    }

    private Set<Task> resolve(List<UUID> taskIds) {
        if (taskIds == null || taskIds.isEmpty()) {
            throw new ConflictException("A note has to be attached to at least one task");
        }
        Set<Task> resolved = new LinkedHashSet<>();
        taskIds.forEach(id -> resolved.add(catalog.requireTask(id)));
        return resolved;
    }

    private String body(DocumentRequest request) {
        return request.bodyMarkdown() == null ? "" : request.bodyMarkdown();
    }

    private Document require(UUID id) {
        return documents.findById(id).orElseThrow(() -> new NotFoundException("Document", id));
    }

    private List<DocumentResponse> map(List<Document> found) {
        return found.stream().map(DocumentResponse::of).toList();
    }
}
