package dev.rekall.domain.repository;

import dev.rekall.domain.RunQueue;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.Optional;
import java.util.UUID;

public interface RunQueueRepository extends JpaRepository<RunQueue, UUID> {

    Optional<RunQueue> findFirstByOrderByCreatedAtAsc();
}
