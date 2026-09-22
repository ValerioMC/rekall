package dev.rekall.domain.repository;

import dev.rekall.domain.Wrapup;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface WrapupRepository extends JpaRepository<Wrapup, UUID> {

    Optional<Wrapup> findByTaskId(UUID taskId);

    List<Wrapup> findAllByOrderByUpdatedAtDesc();

    /** Wrapups whose body holds the term, newest first. The caller escapes the term. */
    @Query("""
           SELECT w FROM Wrapup w
           WHERE LOWER(w.bodyMarkdown) LIKE LOWER(CONCAT('%', :term, '%')) ESCAPE '\\'
           ORDER BY w.updatedAt DESC
           """)
    List<Wrapup> search(@Param("term") String term, Pageable page);
}
