package dev.rekall.api.controller;

import dev.rekall.api.dto.ApiDtos.CommitReferenceDiffResponse;
import dev.rekall.api.dto.ApiDtos.CommitReferenceRequest;
import dev.rekall.api.dto.ApiDtos.PickedCommitReferenceRequest;
import dev.rekall.domain.commit.CommitReferenceService;
import dev.rekall.domain.commit.CommitReferenceView;
import dev.rekall.domain.commit.RecentCommitView;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.UUID;

/**
 * The console's commit controls: log a task's tip commit, pick one from its recent log, or paste a
 * hash — the same logic {@code rekall_record_commit} uses.
 */
@RestController
@RequestMapping("/api")
@RequiredArgsConstructor
public class CommitReferenceController {

    private final CommitReferenceService commitReferences;

    @GetMapping("/commit-references")
    public List<CommitReferenceView> list() {
        return commitReferences.findAll();
    }

    @PostMapping("/tasks/{taskId}/commit-references/latest")
    public CommitReferenceView recordLatest(
            @PathVariable UUID taskId, @RequestBody(required = false) CommitReferenceRequest request) {
        UUID stepId = request == null ? null : request.stepId();
        return commitReferences.recordLatestCommit(taskId, stepId);
    }

    /** What the picker lists: the newest commits of the task's project folder, without their diffs. */
    @GetMapping("/tasks/{taskId}/recent-commits")
    public List<RecentCommitView> recentCommits(@PathVariable UUID taskId) {
        return commitReferences.recentCommits(taskId);
    }

    /** A commit chosen in the picker or pasted by hand, logged by its hash. */
    @PostMapping("/tasks/{taskId}/commit-references")
    public CommitReferenceView record(
            @PathVariable UUID taskId, @Valid @RequestBody PickedCommitReferenceRequest request) {
        return commitReferences.recordCommit(taskId, request.stepId(), request.commitHash());
    }

    @GetMapping("/commit-references/{id}/diff")
    public CommitReferenceDiffResponse diff(@PathVariable UUID id) {
        return new CommitReferenceDiffResponse(commitReferences.diffFor(id));
    }

    @DeleteMapping("/commit-references/{id}")
    public void delete(@PathVariable UUID id) {
        commitReferences.delete(id);
    }
}
