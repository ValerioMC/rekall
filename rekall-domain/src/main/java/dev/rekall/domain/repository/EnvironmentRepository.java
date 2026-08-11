package dev.rekall.domain.repository;

import dev.rekall.domain.Environment;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface EnvironmentRepository extends JpaRepository<Environment, UUID> {

    Optional<Environment> findByLabelIgnoreCase(String label);

    List<Environment> findAllByOrderByLabelAsc();
}
