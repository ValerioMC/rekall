package dev.rekall.domain.claude;

import dev.rekall.domain.Task;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.ClaudeMessageRepository;
import dev.rekall.domain.repository.ClaudeSessionRepository;
import dev.rekall.domain.repository.TaskRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * The persistence side of hosted Claude sessions: it owns the {@code claude_session} and
 * {@code claude_message} rows and the rules on them, and knows nothing about processes. The
 * process itself is {@code ClaudeProcessManager}'s, in the app module.
 *
 * <p>There is no MCP path to any of this. A session is started and steered from the console.
 */
@Service
@RequiredArgsConstructor
public class ClaudeSessionService {

    private static final List<ClaudeSessionStatus> LIVE_STATUSES =
            List.of(ClaudeSessionStatus.STARTING, ClaudeSessionStatus.WORKING, ClaudeSessionStatus.READY);

    private final TaskRepository tasks;
    private final ClaudeSessionRepository sessions;
    private final ClaudeMessageRepository messages;

    @Transactional(readOnly = true)
    public List<ClaudeSessionView> findAll() {
        return sessions.findAllByOrderByStartedAtDesc().stream().map(ClaudeSessionView::of).toList();
    }

    @Transactional(readOnly = true)
    public List<ClaudeSessionView> findByTask(UUID taskId) {
        return sessions.findByTaskIdOrderByStartedAtDesc(taskId).stream().map(ClaudeSessionView::of).toList();
    }

    @Transactional(readOnly = true)
    public Optional<ClaudeSessionView> find(UUID sessionId) {
        return sessions.findById(sessionId).map(ClaudeSessionView::of);
    }

    @Transactional(readOnly = true)
    public List<ClaudeMessageView> transcript(UUID sessionId) {
        return messages.findBySessionIdOrderBySeqAsc(sessionId).stream().map(ClaudeMessageView::of).toList();
    }

    /**
     * Create the row for a new session. The process is not started here: the caller does that
     * and walks the status forward as {@code claude} reports in.
     */
    @Transactional
    public ClaudeSessionView open(UUID taskId, UUID stepId, boolean skipPermissions) {
        Task task = tasks.findById(taskId)
                .orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));

        String folder = task.getProject().getRepoFolder();
        if (folder == null || folder.isBlank()) {
            throw new IllegalArgumentException(
                    "Set this project's folder on its page before running a session here.");
        }

        String anchors = "project:%s task:%s".formatted(task.getProject().getLabel(), task.getLabel());
        ClaudeSession session = sessions.saveAndFlush(
                new ClaudeSession(task, stepId, anchors, folder.strip(), skipPermissions));
        write(session, ClaudeMessageRole.SYSTEM, "Session opened. Loading " + anchors + " with /rk.", null, null);
        return ClaudeSessionView.of(session);
    }

    @Transactional
    public ClaudeMessageView append(UUID sessionId, ClaudeMessageRole role, String content, String toolName, String meta) {
        ClaudeSession session = require(sessionId);
        ClaudeMessage saved = write(session, role, content, toolName, meta);
        session.touch();
        return ClaudeMessageView.of(saved);
    }

    @Transactional
    public void attachCliSession(UUID sessionId, String cliSessionId) {
        require(sessionId).attachCliSession(cliSessionId);
    }

    @Transactional
    public Optional<ClaudeSessionView> markStatus(UUID sessionId, ClaudeSessionStatus status) {
        return sessions.findById(sessionId).map(session -> {
            session.markStatus(status);
            return ClaudeSessionView.of(session);
        });
    }

    @Transactional
    public Optional<ClaudeSessionView> end(UUID sessionId, ClaudeSessionStatus terminal, String detail, Integer exitCode) {
        return sessions.findById(sessionId).map(session -> {
            session.end(terminal, trim(detail, 2_000), exitCode);
            return ClaudeSessionView.of(session);
        });
    }

    @Transactional
    public void delete(UUID sessionId) {
        sessions.findById(sessionId).ifPresent(sessions::delete);
    }

    /**
     * Nothing that reports a status a process would still be attached to can be right after a
     * restart: the process died with the JVM. Mark every such row EXITED so the console shows
     * history rather than a session it can no longer reach. Returns how many were swept.
     */
    @Transactional
    public int recoverOrphans() {
        List<ClaudeSession> orphans = sessions.findByStatusIn(LIVE_STATUSES);
        for (ClaudeSession orphan : orphans) {
            orphan.end(ClaudeSessionStatus.EXITED, "The server restarted while this session was open.", null);
        }
        return orphans.size();
    }

    private ClaudeMessage write(ClaudeSession session, ClaudeMessageRole role, String content, String toolName, String meta) {
        int seq = (int) messages.countBySessionId(session.getId());
        return messages.saveAndFlush(new ClaudeMessage(
                session, seq, role, trim(content, ClaudeMessage.MAX_CONTENT), trim(toolName, 120), trim(meta, 8_000)));
    }

    private ClaudeSession require(UUID sessionId) {
        return sessions.findById(sessionId)
                .orElseThrow(() -> new UnknownAnchorException("No Claude session with id " + sessionId));
    }

    private static String trim(String value, int max) {
        if (value == null) {
            return null;
        }
        if (value.length() <= max) {
            return value;
        }
        String cut = value.substring(0, Math.max(0, max - 20));
        return cut + "\n… (truncated)";
    }
}
