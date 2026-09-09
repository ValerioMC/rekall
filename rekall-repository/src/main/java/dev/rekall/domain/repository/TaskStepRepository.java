package dev.rekall.domain.repository;

import dev.rekall.domain.TaskStep;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.UUID;

public interface TaskStepRepository extends JpaRepository<TaskStep, UUID> {

    List<TaskStep> findByTaskIdOrderByPositionAsc(UUID taskId);

    List<TaskStep> findAllByOrderByTaskIdAscPositionAsc();
}
