package dev.rekall.domain.repository;

import dev.rekall.domain.claude.ClaudeMessage;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.UUID;

public interface ClaudeMessageRepository extends JpaRepository<ClaudeMessage, UUID> {

    List<ClaudeMessage> findBySessionIdOrderBySeqAsc(UUID sessionId);

    long countBySessionId(UUID sessionId);
}
