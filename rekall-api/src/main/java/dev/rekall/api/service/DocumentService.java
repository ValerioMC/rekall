package dev.rekall.api.service;

import dev.rekall.api.dto.ApiDtos.DocumentRequest;
import dev.rekall.api.dto.ApiDtos.DocumentResponse;
import dev.rekall.domain.Document;
import dev.rekall.domain.Environment;
import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.repository.DocumentRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.UUID;

/** The markdown notes attached to a record. */
@Service
@RequiredArgsConstructor
public class DocumentService {

    private final DocumentRepository documents;
    private final CatalogService catalog;

    @Transactional(readOnly = true)
    public List<DocumentResponse> listFor(UUID projectId, UUID taskId, UUID environmentId) {
        if (taskId != null) {
            return map(documents.findByTaskIdOrderByPositionAsc(taskId));
        }
        if (projectId != null) {
            return map(documents.findByProjectIdOrderByPositionAsc(projectId));
        }
        if (environmentId != null) {
            return map(documents.findByEnvironmentIdOrderByPositionAsc(environmentId));
        }
        throw new NotFoundException("One of projectId, taskId or environmentId is required");
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
        attach(document, request);
        return DocumentResponse.of(documents.save(document));
    }

    @Transactional
    public DocumentResponse update(UUID id, DocumentRequest request) {
        Document document = require(id);
        document.setTitle(request.title());
        document.setKind(request.kind());
        document.setBodyMarkdown(body(request));
        return DocumentResponse.of(document);
    }

    @Transactional
    public void delete(UUID id) {
        documents.delete(require(id));
    }

    /**
     * Ownership is set through the owning entity so that both sides of the association agree.
     * Assigning the field directly would leave the in-memory parent's list stale for the rest
     * of the transaction.
     */
    private void attach(Document document, DocumentRequest request) {
        long owners = count(request.projectId()) + count(request.taskId()) + count(request.environmentId());
        if (owners != 1) {
            throw new ConflictException("A document belongs to exactly one project, task or environment");
        }
        if (request.taskId() != null) {
            Task task = catalog.requireTask(request.taskId());
            document.setPosition(task.getDocuments().size());
            task.addDocument(document);
            return;
        }
        if (request.projectId() != null) {
            Project project = catalog.requireProject(request.projectId());
            document.setPosition(project.getDocuments().size());
            project.addDocument(document);
            return;
        }
        Environment environment = catalog.requireEnvironment(request.environmentId());
        document.setPosition(environment.getDocuments().size());
        environment.addDocument(document);
    }

    private int count(UUID id) {
        return id == null ? 0 : 1;
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
