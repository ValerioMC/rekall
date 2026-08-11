package dev.rekall.meta.repository;

import dev.rekall.meta.domain.MetaTable;
import org.springframework.data.jpa.repository.EntityGraph;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface MetaTableRepository extends JpaRepository<MetaTable, UUID> {

    @EntityGraph(attributePaths = "fields")
    Optional<MetaTable> findByPhysicalName(String physicalName);

    /**
     * Always used in place of {@code findById}.
     *
     * <p>The API maps entities to DTOs after the service transaction has closed, and
     * {@code open-in-view} is off, so a lazily loaded field collection fails at serialisation
     * time rather than at query time. Fetching the fields is what every caller wants anyway:
     * an entity without its fields is not useful to anything here.
     */
    @EntityGraph(attributePaths = "fields")
    Optional<MetaTable> findWithFieldsById(UUID id);

    boolean existsByPhysicalName(String physicalName);

    /**
     * Loads every entity with its fields in one query. Used by the schema registry, which
     * rebuilds its whole view rather than loading entities one at a time.
     */
    @EntityGraph(attributePaths = "fields")
    @Query("select distinct t from MetaTable t order by t.label")
    List<MetaTable> findAllWithFields();

    /**
     * Case-insensitive lookup by physical name, label or alias. Lets a caller resolve the name
     * a user typed without forcing an exact physical name.
     */
    @Query(
            """
            select distinct t from MetaTable t
            where lower(t.physicalName) = lower(:name)
               or lower(t.label) = lower(:name)
               or lower(t.labelPlural) = lower(:name)
            """)
    List<MetaTable> findByAnyName(String name);
}
