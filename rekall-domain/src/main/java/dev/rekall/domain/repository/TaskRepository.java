package dev.rekall.domain.repository;

import dev.rekall.domain.Task;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface TaskRepository extends JpaRepository<Task, UUID> {

    List<Task> findByLabelIgnoreCase(String label);

    Optional<Task> findByProjectLabelIgnoreCaseAndLabelIgnoreCase(String projectLabel, String label);

    List<Task> findByProjectIdOrderByLabelAsc(UUID projectId);

    List<Task> findAllByOrderByProjectLabelAscLabelAsc();
}
