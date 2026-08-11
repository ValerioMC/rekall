package dev.rekall.meta.repository;

import dev.rekall.meta.domain.Document;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Modifying;
import org.springframework.data.jpa.repository.Query;

import java.util.List;
import java.util.UUID;

public interface DocumentRepository extends JpaRepository<Document, UUID> {

    List<Document> findByEntityNameAndRecordIdOrderByPositionAsc(String entityName, UUID recordId);

    List<Document> findByEntityNameOrderByPositionAsc(String entityName);

    long countByEntityNameAndRecordId(String entityName, UUID recordId);

    /**
     * Called when an entity is dropped. The database cannot cascade here: the reference into
     * {@code rekall_data} is soft by necessity, so the cleanup is explicit.
     */
    @Modifying
    @Query("delete from Document d where d.entityName = :entityName")
    int deleteByEntityName(String entityName);

    @Modifying
    @Query("delete from Document d where d.entityName = :entityName and d.recordId = :recordId")
    int deleteByEntityNameAndRecordId(String entityName, UUID recordId);
}
