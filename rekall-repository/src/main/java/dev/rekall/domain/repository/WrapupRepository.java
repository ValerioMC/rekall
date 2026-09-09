package dev.rekall.domain.repository;

import dev.rekall.domain.Wrapup;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface WrapupRepository extends JpaRepository<Wrapup, UUID> {

    Optional<Wrapup> findByTaskId(UUID taskId);

    List<Wrapup> findAllByOrderByUpdatedAtDesc();
}
