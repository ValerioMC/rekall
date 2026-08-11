package dev.rekall.meta.repository;

import dev.rekall.meta.domain.DdlLog;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.UUID;

public interface DdlLogRepository extends JpaRepository<DdlLog, UUID> {

    List<DdlLog> findByPlanIdOrderBySequenceAsc(UUID planId);

    /** Full history in execution order. This is the recipe to rebuild {@code rekall_data}. */
    List<DdlLog> findAllByOrderByAppliedAtAscSequenceAsc();
}
