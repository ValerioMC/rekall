package dev.rekall.domain.repository;

import dev.rekall.domain.TimeEntry;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface TimeEntryRepository extends JpaRepository<TimeEntry, UUID> {

    /**
     * The open session on this task, if any — an optional rather than a list because only one
     * may be open per task at a time, and {@code TimeEntryService} is what keeps that true.
     * Different tasks may each have one open at once.
     */
    Optional<TimeEntry> findByTaskIdAndStoppedAtIsNull(UUID taskId);

    /** Every session, most recently started first — the order the recap reads them in. */
    List<TimeEntry> findAllByOrderByStartedAtDesc();
}
