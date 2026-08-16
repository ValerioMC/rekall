package dev.rekall.domain.repository;

import dev.rekall.domain.TimeEntry;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface TimeEntryRepository extends JpaRepository<TimeEntry, UUID> {

    /**
     * The one session, anywhere, that has not been stopped yet — an optional rather than a
     * list because only one may be open at a time, and {@code TimeEntryService} is what keeps
     * that true.
     */
    Optional<TimeEntry> findByStoppedAtIsNull();

    /** Every session, most recently started first — the order the recap reads them in. */
    List<TimeEntry> findAllByOrderByStartedAtDesc();
}
