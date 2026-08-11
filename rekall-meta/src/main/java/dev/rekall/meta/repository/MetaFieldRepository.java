package dev.rekall.meta.repository;

import dev.rekall.meta.domain.MetaField;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.Optional;
import java.util.UUID;

public interface MetaFieldRepository extends JpaRepository<MetaField, UUID> {

    Optional<MetaField> findByMetaTableIdAndColumnName(UUID metaTableId, String columnName);
}
