package dev.rekall.domain.repository;

import dev.rekall.domain.TaskStep;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

import java.util.List;
import java.util.UUID;

public interface TaskStepRepository extends JpaRepository<TaskStep, UUID> {

    List<TaskStep> findByTaskIdOrderByPositionAsc(UUID taskId);

    List<TaskStep> findAllByOrderByTaskIdAscPositionAsc();

    /** Steps whose title or detail holds the term, newest first. The caller escapes the term. */
    @Query("""
           SELECT s FROM TaskStep s
           WHERE LOWER(s.title) LIKE LOWER(CONCAT('%', :term, '%')) ESCAPE '\\'
              OR LOWER(s.bodyMarkdown) LIKE LOWER(CONCAT('%', :term, '%')) ESCAPE '\\'
           ORDER BY s.updatedAt DESC
           """)
    List<TaskStep> search(@Param("term") String term, Pageable page);
}
