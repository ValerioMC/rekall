package dev.rekall.domain.repository;

import dev.rekall.domain.Document;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

import java.util.List;
import java.util.UUID;

public interface DocumentRepository extends JpaRepository<Document, UUID> {

    List<Document> findByTasksIdOrderByTitleAsc(UUID taskId);

    List<Document> findAllByOrderByUpdatedAtDesc();

    @Query("""
           SELECT DISTINCT d FROM Document d
           JOIN d.tasks t
           WHERE t.project.id = :projectId
           ORDER BY d.updatedAt DESC
           """)
    List<Document> findByProject(@Param("projectId") UUID projectId);

    @Query("SELECT d FROM Document d WHERE d.tasks IS EMPTY")
    List<Document> findOrphans();

    @Query("""
           SELECT d FROM Document d
           WHERE LOWER(d.title) LIKE LOWER(CONCAT('%', :term, '%'))
              OR LOWER(d.bodyMarkdown) LIKE LOWER(CONCAT('%', :term, '%'))
           ORDER BY d.title ASC
           """)
    List<Document> search(@Param("term") String term);

    /** Notes whose title or body holds the term, newest first. The caller escapes the term. */
    @Query("""
           SELECT d FROM Document d
           WHERE LOWER(d.title) LIKE LOWER(CONCAT('%', :term, '%')) ESCAPE '\\'
              OR LOWER(d.bodyMarkdown) LIKE LOWER(CONCAT('%', :term, '%')) ESCAPE '\\'
           ORDER BY d.updatedAt DESC
           """)
    List<Document> searchText(@Param("term") String term, Pageable page);

    /** Notes whose id starts with the prefix: how a {@code note:} anchor, an abbreviated id, resolves. */
    @Query("SELECT d FROM Document d WHERE LOWER(CAST(d.id AS String)) LIKE CONCAT(:prefix, '%')")
    List<Document> findByIdPrefix(@Param("prefix") String prefix);
}
