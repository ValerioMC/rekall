package dev.rekall.domain.repository;

import dev.rekall.domain.RunQueueItem;
import org.springframework.data.jpa.repository.EntityGraph;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.UUID;

public interface RunQueueItemRepository extends JpaRepository<RunQueueItem, UUID> {

    @EntityGraph(attributePaths = {"task", "task.project"})
    List<RunQueueItem> findAllByOrderByPositionAsc();
}
