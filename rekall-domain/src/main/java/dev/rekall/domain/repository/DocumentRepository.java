package dev.rekall.domain.repository;

import dev.rekall.domain.Document;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

import java.util.List;
import java.util.UUID;

public interface DocumentRepository extends JpaRepository<Document, UUID> {

    List<Document> findByTasksIdOrderByTitleAsc(UUID taskId);

    /**
     * Every note, most recently written first.
     *
     * <p>Ordered by {@code updatedAt} because the screen that reads this is the one asking
     * "what was I working on", and the answer to that is chronological rather than alphabetical.
     */
    List<Document> findAllByOrderByUpdatedAtDesc();

    /**
     * The notes of every task on one project.
     *
     * <p>{@code DISTINCT} because a note attached to three tasks of the same project would
     * otherwise come back three times, which is the join showing through where it should not.
     */
    @Query("""
           SELECT DISTINCT d FROM Document d
           JOIN d.tasks t
           WHERE t.project.id = :projectId
           ORDER BY d.updatedAt DESC
           """)
    List<Document> findByProject(@Param("projectId") UUID projectId);

    /** Notes left on no task at all, which the model has no way to reach and no reason to keep. */
    @Query("SELECT d FROM Document d WHERE d.tasks IS EMPTY")
    List<Document> findOrphans();

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
