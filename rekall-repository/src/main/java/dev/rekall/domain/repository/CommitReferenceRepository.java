package dev.rekall.domain.repository;

import dev.rekall.domain.CommitReference;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.UUID;

public interface CommitReferenceRepository extends JpaRepository<CommitReference, UUID> {

    List<CommitReference> findByTaskIdAndCommitHash(UUID taskId, String commitHash);

    List<CommitReference> findAllByOrderByCreatedAtDesc();

    /** The rows a task hands to {@code rekall_context}, in the order they were logged. */
    List<CommitReference> findByTaskIdAndInContextTrueOrderByCreatedAtAsc(UUID taskId);
}
