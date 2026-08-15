package dev.rekall.domain.repository;

import dev.rekall.domain.Project;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface ProjectRepository extends JpaRepository<Project, UUID> {

    /**
     * Resolves the value side of a {@code project:vega} anchor.
     *
     * <p>On the label and never on the title: the anchor is an identifier, and a title is free to
     * be rewritten without invalidating what someone wrote in a slash command last month.
     *
     * <p>A list, not an optional: labels are unique per company, so the same one can exist on two
     * companies and the caller has to decide what an ambiguous anchor means.
     */
    List<Project> findByLabelIgnoreCase(String label);

    Optional<Project> findByCompanyNameIgnoreCaseAndLabelIgnoreCase(String companyName, String label);

    List<Project> findByCompanyIdOrderByLabelAsc(UUID companyId);

    List<Project> findAllByOrderByCompanyNameAscLabelAsc();
}
