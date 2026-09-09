package dev.rekall.claude;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import dev.rekall.api.service.ConflictException;
import dev.rekall.claude.ClaudeStreamMapper.Entry;
import dev.rekall.claude.ClaudeStreamMapper.Mapped;
import dev.rekall.domain.claude.ClaudeMessageRole;
import dev.rekall.domain.claude.ClaudeMessageView;
import dev.rekall.domain.claude.ClaudeSessionService;
import dev.rekall.domain.claude.ClaudeSessionStatus;
import dev.rekall.domain.claude.ClaudeSessionView;
import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

/**
 * Owns the {@code claude} processes behind hosted sessions: one per session, several at once.
 *
 * <p>Each is started with {@code --input-format stream-json --output-format stream-json}, so the
 * session stays open on stdin and every prompt after the first is another line written to it.
 * A virtual thread reads the NDJSON coming back, {@link ClaudeStreamMapper} turns it into
 * transcript entries, and {@link ClaudeSessionService} persists them while {@link
 * ClaudeSessionStream} pushes them to whichever console is watching.
 *
 * <p>What keeps stray processes from piling up: a hard cap on how many run at once, an idle
 * sweep that closes a session left untouched too long, and a shutdown hook that kills the rest.
 * A restart is caught on the way back up by {@code ClaudeSessionRecovery}.
 */
@Component
@Slf4j
public class ClaudeProcessManager {

    private final ClaudeSessionService sessions;
    private final ClaudeStreamMapper mapper;
    private final ClaudeSessionStream stream;
    private final ClaudeCli cli;
    private final ObjectMapper json = new ObjectMapper();

    private final int maxLive;
    private final long idleMinutes;
    private final long sweepMinutes;

    private final ConcurrentHashMap<UUID, Live> live = new ConcurrentHashMap<>();
    private ScheduledExecutorService reaper;

    public ClaudeProcessManager(
            ClaudeSessionService sessions,
            ClaudeStreamMapper mapper,
            ClaudeSessionStream stream,
            ClaudeCli cli,
            @Value("${rekall.claude.max-sessions:8}") int maxLive,
            @Value("${rekall.claude.idle-minutes:120}") long idleMinutes,
            @Value("${rekall.claude.sweep-minutes:5}") long sweepMinutes) {
        this.sessions = sessions;
        this.mapper = mapper;
        this.stream = stream;
        this.cli = cli;
        this.maxLive = maxLive;
        this.idleMinutes = idleMinutes;
        this.sweepMinutes = sweepMinutes;
    }

    private static final class Live {
        private final Process process;
        private final BufferedWriter stdin;
        private final StringBuilder stderrTail = new StringBuilder();
        private volatile boolean closing;

        private Live(Process process, BufferedWriter stdin) {
            this.process = process;
            this.stdin = stdin;
        }
    }

    // ---------------------------------------------------------------- lifecycle

    @PostConstruct
    void startReaper() {
        reaper = Executors.newSingleThreadScheduledExecutor(runnable -> {
            Thread thread = new Thread(runnable, "claude-reaper");
            thread.setDaemon(true);
            return thread;
        });
        reaper.scheduleWithFixedDelay(this::sweepIdle, sweepMinutes, sweepMinutes, TimeUnit.MINUTES);
    }

    @PreDestroy
    void shutdown() {
        if (reaper != null) {
            reaper.shutdownNow();
        }
        for (UUID id : List.copyOf(live.keySet())) {
            Live handle = live.remove(id);
            if (handle == null) {
                continue;
            }
            handle.closing = true;
            handle.process.destroy();
            try {
                sessions.end(id, ClaudeSessionStatus.EXITED, "The app was shutting down.", null);
            } catch (RuntimeException ignored) {
                // Best effort: the startup sweep is the backstop.
            }
        }
    }

    // ---------------------------------------------------------------- commands

    public ClaudeSessionView start(
            UUID taskId, UUID stepId, boolean skipPermissions, String model, String effort) {
        if (live.size() >= maxLive) {
            throw new ConflictException(
                    "Rekall is already running " + maxLive + " sessions. Stop one before starting another.");
        }

        String chosenModel = normaliseModel(model);
        String chosenEffort = normaliseEffort(effort);
        ClaudeSessionView view = sessions.open(taskId, stepId, skipPermissions, chosenModel, chosenEffort);
        UUID id = view.id();

        Path directory = Path.of(view.workingDir());
        if (!Files.isDirectory(directory)) {
            fail(id, "The folder " + view.workingDir() + " is not on this machine.");
            throw new ConflictException(
                    "This project's folder (" + view.workingDir() + ") is not there. Set it again on the project page.");
        }

        Path binary = cli.locate().orElse(null);
        if (binary == null) {
            fail(id, "The claude command line tool was not found.");
            throw new ConflictException(
                    "Claude Code's command line tool isn't on this machine. Install it, then try again.");
        }

        List<String> command = new ArrayList<>(List.of(
                binary.toString(), "--print", "--verbose",
                "--input-format", "stream-json", "--output-format", "stream-json"));
        if (skipPermissions) {
            command.add("--dangerously-skip-permissions");
        }
        if (chosenModel != null) {
            command.add("--model");
            command.add(chosenModel);
        }
        if (chosenEffort != null) {
            command.add("--effort");
            command.add(chosenEffort);
        }

        ProcessBuilder builder = new ProcessBuilder(command).directory(directory.toFile());
        builder.environment().putAll(cli.environment());

        Process process;
        try {
            process = builder.start();
        } catch (IOException startFailed) {
            fail(id, "Could not start claude: " + startFailed.getMessage());
            throw new ConflictException("Could not start claude: " + startFailed.getMessage());
        }

        Live handle = new Live(process,
                new BufferedWriter(new OutputStreamWriter(process.getOutputStream(), StandardCharsets.UTF_8)));
        live.put(id, handle);

        Thread.ofVirtual().name("claude-stdout-" + id).start(() -> pumpStdout(id, process));
        Thread.ofVirtual().name("claude-stderr-" + id).start(() -> drainStderr(handle));
        process.onExit().thenRun(() -> onExit(id));

        try {
            writeLine(handle, "/rk " + view.anchors());
        } catch (RuntimeException openingFailed) {
            stop(id, "Could not send the opening prompt: " + openingFailed.getMessage());
            throw openingFailed;
        }

        sessions.markStatus(id, ClaudeSessionStatus.WORKING).ifPresent(this::emitStatus);
        return sessions.find(id).orElse(view);
    }

    public ClaudeMessageView prompt(UUID sessionId, String text) {
        String trimmed = text == null ? "" : text.strip();
        if (trimmed.isEmpty()) {
            throw new IllegalArgumentException("A prompt can't be empty.");
        }
        Live handle = live.get(sessionId);
        if (handle == null) {
            throw new ConflictException("This session has ended. Start a new one to keep going.");
        }

        ClaudeMessageView echoed = sessions.append(sessionId, ClaudeMessageRole.USER, trimmed, null, null);
        stream.emit(sessionId, "message", echoed);
        writeLine(handle, trimmed);
        sessions.markStatus(sessionId, ClaudeSessionStatus.WORKING).ifPresent(this::emitStatus);
        return echoed;
    }

    public ClaudeSessionView stop(UUID sessionId, String reason) {
        Live handle = live.remove(sessionId);
        if (handle != null) {
            handle.closing = true;
            try {
                handle.stdin.close();
            } catch (IOException ignored) {
                // Already gone.
            }
            handle.process.destroy();
            try {
                if (!handle.process.waitFor(3, TimeUnit.SECONDS)) {
                    handle.process.destroyForcibly();
                }
            } catch (InterruptedException interrupted) {
                Thread.currentThread().interrupt();
                handle.process.destroyForcibly();
            }
        }
        return sessions.end(sessionId, ClaudeSessionStatus.EXITED, reason, null)
                .map(this::announceEnded)
                .or(() -> sessions.find(sessionId))
                .orElse(null);
    }

    public void delete(UUID sessionId) {
        stop(sessionId, "Removed from the console.");
        sessions.delete(sessionId);
    }

    public boolean isLive(UUID sessionId) {
        return live.containsKey(sessionId);
    }

    public int liveCount() {
        return live.size();
    }

    // ---------------------------------------------------------------- process io

    private void pumpStdout(UUID sessionId, Process process) {
        try (BufferedReader reader = new BufferedReader(
                new InputStreamReader(process.getInputStream(), StandardCharsets.UTF_8))) {
            String line;
            while ((line = reader.readLine()) != null) {
                handleLine(sessionId, line);
            }
        } catch (IOException ended) {
            log.debug("stdout for session {} closed: {}", sessionId, ended.getMessage());
        } catch (RuntimeException gone) {
            log.debug("stopped reading session {}: {}", sessionId, gone.getMessage());
        }
    }

    private void handleLine(UUID sessionId, String line) {
        Mapped mapped = mapper.map(line);
        if (mapped.cliSessionId() != null) {
            sessions.attachCliSession(sessionId, mapped.cliSessionId());
        }
        if (mapped.model() != null) {
            sessions.resolveModel(sessionId, mapped.model()).ifPresent(this::emitStatus);
        }
        for (Entry entry : mapped.entries()) {
            ClaudeMessageView view =
                    sessions.append(sessionId, entry.role(), entry.content(), entry.toolName(), entry.meta());
            stream.emit(sessionId, "message", view);
        }
        if (mapped.turnComplete()) {
            sessions.markStatus(sessionId, ClaudeSessionStatus.READY).ifPresent(this::emitStatus);
        }
    }

    private void drainStderr(Live handle) {
        try (BufferedReader reader = new BufferedReader(
                new InputStreamReader(handle.process.getErrorStream(), StandardCharsets.UTF_8))) {
            String line;
            while ((line = reader.readLine()) != null) {
                if (handle.stderrTail.length() < 8_000) {
                    handle.stderrTail.append(line).append('\n');
                }
            }
        } catch (IOException ignored) {
            // The stream closed with the process.
        }
    }

    private void onExit(UUID sessionId) {
        Live handle = live.remove(sessionId);
        if (handle == null) {
            return;
        }
        int code = handle.process.exitValue();
        ClaudeSessionStatus terminal = code == 0 ? ClaudeSessionStatus.EXITED : ClaudeSessionStatus.FAILED;
        String stderr = handle.stderrTail.toString().strip();
        String detail = code == 0
                ? "The session ended."
                : stderr.isBlank() ? "claude exited with code " + code : stderr;
        sessions.end(sessionId, terminal, detail, code).ifPresent(this::announceEnded);
    }

    private void writeLine(Live handle, String text) {
        ObjectNode message = json.createObjectNode();
        message.put("type", "user");
        ObjectNode inner = message.putObject("message");
        inner.put("role", "user");
        ArrayNode content = inner.putArray("content");
        ObjectNode block = content.addObject();
        block.put("type", "text");
        block.put("text", text);
        synchronized (handle) {
            try {
                handle.stdin.write(json.writeValueAsString(message));
                handle.stdin.write("\n");
                handle.stdin.flush();
            } catch (IOException refused) {
                throw new ConflictException("The session is no longer accepting input.");
            }
        }
    }

    // ---------------------------------------------------------------- helpers

    private void sweepIdle() {
        try {
            Instant cutoff = Instant.now().minus(Duration.ofMinutes(idleMinutes));
            for (UUID id : List.copyOf(live.keySet())) {
                sessions.find(id).ifPresent(view -> {
                    if (view.lastActivityAt().isBefore(cutoff)) {
                        log.info("Closing idle Claude session {} ({})", id, view.anchor());
                        stop(id, "Closed after " + idleMinutes + " minutes with no activity.");
                    }
                });
            }
        } catch (RuntimeException sweepFailed) {
            log.warn("Idle sweep failed: {}", sweepFailed.getMessage());
        }
    }

    private static final Set<String> MODEL_ALIASES = Set.of("opus", "sonnet", "haiku", "fable");
    private static final Set<String> EFFORT_LEVELS = Set.of("low", "medium", "high", "xhigh", "max");

    /**
     * What to hand {@code claude --model}, or null to leave the account default in place. Only the
     * aliases the settings offer are accepted; each is Claude Code's own name for the latest
     * model of that family, so no version is pinned. Anything else (including "default") is
     * treated as no choice, so an unexpected value can never end up as a process argument.
     */
    private String normaliseModel(String model) {
        if (model == null) {
            return null;
        }
        String trimmed = model.strip().toLowerCase(Locale.ROOT);
        return MODEL_ALIASES.contains(trimmed) ? trimmed : null;
    }

    /**
     * What to hand {@code claude --effort}, or null to leave it unset. Same gate as the model:
     * only the five levels the CLI defines pass, everything else (including "default") is no
     * choice.
     */
    private String normaliseEffort(String effort) {
        if (effort == null) {
            return null;
        }
        String trimmed = effort.strip().toLowerCase(Locale.ROOT);
        return EFFORT_LEVELS.contains(trimmed) ? trimmed : null;
    }

    private void fail(UUID sessionId, String detail) {
        sessions.end(sessionId, ClaudeSessionStatus.FAILED, detail, null).ifPresent(this::announceEnded);
    }

    private ClaudeSessionView announceEnded(ClaudeSessionView view) {
        emitStatus(view);
        stream.emit(view.id(), "ended", view);
        return view;
    }

    private void emitStatus(ClaudeSessionView view) {
        stream.emit(view.id(), "status", view);
    }
}
