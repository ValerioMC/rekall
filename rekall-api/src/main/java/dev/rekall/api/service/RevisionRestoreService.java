package dev.rekall.api.service;

import dev.rekall.domain.revision.RestoredRevision;
import dev.rekall.domain.revision.TaskRevisionService;
import dev.rekall.domain.revision.TaskRevisionView;
import dev.rekall.domain.wrapup.WrapupService;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.UUID;

/**
 * Writes a kept revision back as the current wrapup or description. It goes through the same
 * write path as any other edit, so the text it replaces becomes a revision in turn and a restore
 * can be undone the same way. It lives here, beside {@link CatalogService}, and not in
 * {@code rekall-service}: writing a task's description is a console write, and the MCP module must
 * not have it on its classpath.
 */
@Service
@RequiredArgsConstructor
public class RevisionRestoreService {

    private final TaskRevisionService revisions;
    private final WrapupService wrapups;
    private final CatalogService catalog;

    @Transactional
    public RestoredRevision restore(UUID taskId, UUID revisionId) {
        TaskRevisionView revision = revisions.find(taskId, revisionId);
        switch (revision.kind()) {
            case WRAPUP -> wrapups.restore(taskId, revision.bodyMarkdown());
            case DESCRIPTION -> catalog.restoreDescription(taskId, revision.bodyMarkdown());
        }
        return new RestoredRevision(taskId, revision.kind(), revision.bodyMarkdown());
    }
}
