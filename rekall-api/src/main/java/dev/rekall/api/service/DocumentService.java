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

/**
 * The markdown notes, and which tasks each one belongs to.
 *
 * <p>A note is attached to at least one task and possibly to many. The lower bound is enforced
 * here rather than in the database, because it is a rule about what the interface may leave
 * behind and not about referential integrity: a note on no task is unreachable, and the model
 * has no screen that could show it again.
 */
@Service
@RequiredArgsConstructor
public class DocumentService {

    private final DocumentRepository documents;
    private final CatalogService catalog;

    /**
     * Notes for one task, for one project, or all of them.
     *
     * <p>Three scopes on one endpoint because they are the same question asked at three widths,
     * and the screen switches between them as you change what you are looking at.
     */
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

    /** Removes the note everywhere. Detaching it from one task is an update, not a delete. */
    @Transactional
    public void delete(UUID id) {
        Document document = require(id);
        Set.copyOf(document.getTasks()).forEach(task -> task.detach(document));
        documents.delete(document);
    }

    /**
     * Brings the set of tasks to exactly what was asked for.
     *
     * <p>Computed as a difference rather than cleared and rebuilt: clearing would delete every
     * join row and insert them again on each save, which churns the order column and shows up
     * as notes reshuffling themselves on tasks nobody touched.
     */
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
