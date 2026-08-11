package dev.rekall.domain.repository;

import dev.rekall.domain.Document;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

import java.util.List;
import java.util.UUID;

public interface DocumentRepository extends JpaRepository<Document, UUID> {

    List<Document> findByProjectIdOrderByPositionAsc(UUID projectId);

    List<Document> findByTaskIdOrderByPositionAsc(UUID taskId);

    List<Document> findByEnvironmentIdOrderByPositionAsc(UUID environmentId);

    /**
     * Substring match rather than a full-text index.
     *
     * <p>PostgreSQL's {@code tsvector} went away with PostgreSQL. On a single user's notes a
     * {@code LIKE} over a few hundred rows is instant, and unlike a stemmer it also finds the
     * middle of {@code /api/v1/pipelines}, which the previous implementation could not.
     */
    @Query("""
           SELECT d FROM Document d
           WHERE LOWER(d.title) LIKE LOWER(CONCAT('%', :term, '%'))
              OR LOWER(d.bodyMarkdown) LIKE LOWER(CONCAT('%', :term, '%'))
           ORDER BY d.title ASC
           """)
    List<Document> search(@Param("term") String term);
}
