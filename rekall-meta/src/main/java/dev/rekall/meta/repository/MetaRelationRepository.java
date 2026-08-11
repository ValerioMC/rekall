package dev.rekall.meta.repository;

import dev.rekall.meta.domain.MetaRelation;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface MetaRelationRepository extends JpaRepository<MetaRelation, UUID> {

    List<MetaRelation> findBySourceTableId(UUID sourceTableId);

    List<MetaRelation> findByTargetTableId(UUID targetTableId);

    Optional<MetaRelation> findBySourceFieldId(UUID sourceFieldId);

    /** Every relation touching the entity, in either direction. Used to block unsafe drops. */
    @Query("select r from MetaRelation r where r.sourceTable.id = :tableId or r.targetTable.id = :tableId")
    List<MetaRelation> findAllTouching(UUID tableId);

    @Query("select r from MetaRelation r join fetch r.sourceTable join fetch r.targetTable")
    List<MetaRelation> findAllWithTables();
}
