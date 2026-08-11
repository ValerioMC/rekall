package dev.rekall.domain.repository;

import dev.rekall.domain.Task;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface TaskRepository extends JpaRepository<Task, UUID> {

    /**
     * A list, not an optional: task names are unique per project, so the same name can exist on
     * two projects and the caller has to decide what an ambiguous anchor means.
     */
    List<Task> findByNameIgnoreCase(String name);

    Optional<Task> findByProjectNameIgnoreCaseAndNameIgnoreCase(String projectName, String name);

    List<Task> findByProjectIdOrderByNameAsc(UUID projectId);
}
