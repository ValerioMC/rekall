package dev.rekall.domain.repository;

import dev.rekall.domain.CommitReference;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.UUID;

public interface CommitReferenceRepository extends JpaRepository<CommitReference, UUID> {

    List<CommitReference> findByTaskIdAndCommitHash(UUID taskId, String commitHash);
}
