package dev.rekall.domain.repository;

import dev.rekall.domain.Wrapup;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface WrapupRepository extends JpaRepository<Wrapup, UUID> {

    /**
     * An optional, not a list: {@code task_id} is unique, so a second row cannot exist for the
     * database to return.
     */
    Optional<Wrapup> findByTaskId(UUID taskId);

    /**
     * Every wrapup, most recently written first.
     *
     * <p>Ordered the way the console reads them, which is the same order the notes list uses:
     * the question being asked is "what did I last leave in a state worth recording".
     */
    List<Wrapup> findAllByOrderByUpdatedAtDesc();
}
