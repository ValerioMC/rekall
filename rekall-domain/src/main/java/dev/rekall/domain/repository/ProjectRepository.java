package dev.rekall.domain.repository;

import dev.rekall.domain.Project;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface ProjectRepository extends JpaRepository<Project, UUID> {

    /** Resolves the value side of a {@code project:stvv} anchor. */
    Optional<Project> findByNameIgnoreCase(String name);

    List<Project> findAllByOrderByNameAsc();
}
