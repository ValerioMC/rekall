package dev.rekall.domain.repository;

import dev.rekall.domain.Task;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface TaskRepository extends JpaRepository<Task, UUID> {

    List<Task> findByLabelIgnoreCase(String label);

    Optional<Task> findByProjectLabelIgnoreCaseAndLabelIgnoreCase(String projectLabel, String label);

    List<Task> findByProjectIdOrderByLabelAsc(UUID projectId);

    List<Task> findAllByOrderByProjectLabelAscLabelAsc();

    /** Tasks whose description holds the term, newest first. {@code %} and {@code _} in the term are escaped by the caller. */
    @Query("""
           SELECT t FROM Task t
           WHERE LOWER(t.description) LIKE LOWER(CONCAT('%', :term, '%')) ESCAPE '\\'
           ORDER BY t.updatedAt DESC
           """)
    List<Task> searchDescriptions(@Param("term") String term, Pageable page);
}
