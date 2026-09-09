package dev.rekall.domain.repository;

import dev.rekall.domain.claude.ClaudeSession;
import dev.rekall.domain.claude.ClaudeSessionStatus;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.UUID;

public interface ClaudeSessionRepository extends JpaRepository<ClaudeSession, UUID> {

    List<ClaudeSession> findAllByOrderByStartedAtDesc();

    List<ClaudeSession> findByTaskIdOrderByStartedAtDesc(UUID taskId);

    List<ClaudeSession> findByStatusIn(List<ClaudeSessionStatus> statuses);
}
