package dev.rekall.domain.repository;

import dev.rekall.domain.Task;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface TaskRepository extends JpaRepository<Task, UUID> {

    /**
     * A list, not an optional: labels are unique per project, so the same one can exist on two
     * projects and the caller has to decide what an ambiguous anchor means.
     */
    List<Task> findByLabelIgnoreCase(String label);

    Optional<Task> findByProjectLabelIgnoreCaseAndLabelIgnoreCase(String projectLabel, String label);

    List<Task> findByProjectIdOrderByLabelAsc(UUID projectId);

    /**
     * Every task, grouped by the project it belongs to.
     *
     * <p>The unscoped list is read by a screen that mixes projects, where {@code findAll} would
     * return whatever order the database happened to produce and reshuffle it between calls.
     */
    List<Task> findAllByOrderByProjectLabelAscLabelAsc();
}
