package dev.rekall.domain.repository;

import dev.rekall.domain.TimeEntry;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface TimeEntryRepository extends JpaRepository<TimeEntry, UUID> {

    Optional<TimeEntry> findByTaskIdAndStoppedAtIsNull(UUID taskId);

    List<TimeEntry> findAllByOrderByStartedAtDesc();
}
