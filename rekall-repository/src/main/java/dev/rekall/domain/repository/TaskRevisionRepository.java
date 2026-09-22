package dev.rekall.domain.repository;

import dev.rekall.domain.RevisionKind;
import dev.rekall.domain.TaskRevision;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface TaskRevisionRepository extends JpaRepository<TaskRevision, UUID> {

    List<TaskRevision> findByTaskIdAndKindOrderByCreatedAtDesc(UUID taskId, RevisionKind kind);

    Optional<TaskRevision> findFirstByTaskIdAndKindOrderByCreatedAtDesc(UUID taskId, RevisionKind kind);

    Optional<TaskRevision> findByIdAndTaskId(UUID id, UUID taskId);
}
