package dev.rekall.domain.repository;

import dev.rekall.domain.TaskStep;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.UUID;

public interface TaskStepRepository extends JpaRepository<TaskStep, UUID> {

    /** The list as it is shown and as it is renumbered: order is part of what a step is. */
    List<TaskStep> findByTaskIdOrderByPositionAsc(UUID taskId);

    /**
     * Every step, in the order the console lists them under their tasks.
     *
     * <p>Read whole the way the wrapups are: the console holds the catalogue in memory, so
     * selecting a task shows its checklist without a round trip.
     */
    List<TaskStep> findAllByOrderByTaskIdAscPositionAsc();
}
