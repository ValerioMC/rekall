package dev.rekall.domain.repository;

import dev.rekall.domain.Project;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface ProjectRepository extends JpaRepository<Project, UUID> {

    List<Project> findByLabelIgnoreCase(String label);

    Optional<Project> findByCompanyNameIgnoreCaseAndLabelIgnoreCase(String companyName, String label);

    List<Project> findByCompanyIdOrderByLabelAsc(UUID companyId);

    List<Project> findAllByOrderByCompanyNameAscLabelAsc();
}
