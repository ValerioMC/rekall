package dev.rekall.api.controller;

import dev.rekall.api.service.RevisionRestoreService;
import dev.rekall.domain.RevisionKind;
import dev.rekall.domain.revision.RestoredRevision;
import dev.rekall.domain.revision.TaskRevisionService;
import dev.rekall.domain.revision.TaskRevisionView;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.UUID;

@RestController
@RequestMapping("/api/tasks/{taskId}/revisions")
@RequiredArgsConstructor
public class TaskRevisionController {

    private final TaskRevisionService revisions;
    private final RevisionRestoreService restorer;

    @GetMapping
    public List<TaskRevisionView> list(@PathVariable UUID taskId, @RequestParam RevisionKind kind) {
        return revisions.list(taskId, kind);
    }

    @PostMapping("/{revisionId}/restore")
    public RestoredRevision restore(@PathVariable UUID taskId, @PathVariable UUID revisionId) {
        return restorer.restore(taskId, revisionId);
    }
}
