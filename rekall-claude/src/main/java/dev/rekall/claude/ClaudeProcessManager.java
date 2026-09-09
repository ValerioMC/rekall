package dev.rekall.claude;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import dev.rekall.common.ConflictException;
import dev.rekall.claude.ClaudeStreamMapper.Entry;
import dev.rekall.claude.ClaudeStreamMapper.Mapped;
import dev.rekall.domain.claude.ClaudeMessageRole;
import dev.rekall.domain.claude.ClaudeMessageView;
import dev.rekall.domain.claude.ClaudeSessionService;
import dev.rekall.domain.claude.ClaudeSessionStatus;
import dev.rekall.domain.claude.ClaudeSessionView;
import dev.rekall.domain.review.TaskReviewService;
import dev.rekall.domain.step.TaskStepService;
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
    private final TaskReviewService taskReview;
    private final TaskStepService taskSteps;
    private final ObjectMapper json = new ObjectMapper();

    private final int maxLive;
    private final long idleMinutes;
    private final long sweepMinutes;

    private final ConcurrentHashMap<UUID, Live> live = new ConcurrentHashMap<>();

    /**
     * Serialises only the spawn decision: "is a process already up for this task? no -> create and
     * register it". Held for the length of a {@link ProcessBuilder#start()}, never across a write
     * to an existing process's stdin, so a wedged session can never make this the thing that holds
     * up graceful shutdown (see application.yaml on the 5s phase budget).
     */
    private final Object spawnGate = new Object();

    private ScheduledExecutorService reaper;

    public ClaudeProcessManager(
            ClaudeSessionService sessions,
            ClaudeStreamMapper mapper,
            ClaudeSessionStream stream,
            ClaudeCli cli,
            TaskReviewService taskReview,
            TaskStepService taskSteps,
            @Value("${rekall.claude.max-sessions:8}") int maxLive,
            @Value("${rekall.claude.idle-minutes:120}") long idleMinutes,
            @Value("${rekall.claude.sweep-minutes:5}") long sweepMinutes) {
        this.sessions = sessions;
        this.mapper = mapper;
        this.stream = stream;
        this.cli = cli;
        this.taskReview = taskReview;
        this.taskSteps = taskSteps;
        this.maxLive = maxLive;
        this.idleMinutes = idleMinutes;
        this.sweepMinutes = sweepMinutes;
    }

    private static final class Live {
        private final Process process;
        private final BufferedWriter stdin;
        private final UUID taskId;
        private volatile UUID stepId;
        private final StringBuilder stderrTail = new StringBuilder();
        private volatile boolean closing;

        private Live(Process process, BufferedWriter stdin, UUID taskId, UUID stepId) {
            this.process = process;
            this.stdin = stdin;
            this.taskId = taskId;
            this.stepId = stepId;
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

    /**
     * One live process per task, the way one terminal window is. The first "Run here" on a task
     * spawns {@code claude} and loads {@code /rk}; every press after that, for any step, is
     * handed to {@link #reuse} so the warm session (and its prompt cache) is kept rather than a
     * second cold process started.
     *
     * <p>Only the spawn is serialised, on {@link #spawnGate}, so two fast clicks cannot both start
     * a process. {@link #reuse} runs outside that lock: if the session ended in the gap it throws
     * a retriable conflict rather than blocking a caller behind a slow stdin write.
     */
    public ClaudeSessionView start(
            UUID taskId, UUID stepId, boolean skipPermissions, String model, String effort) {
        synchronized (spawnGate) {
            if (liveSessionForTask(taskId) == null) {
                return spawn(taskId, stepId, skipPermissions, model, effort);
            }
        }
        UUID liveSessionId = liveSessionForTask(taskId);
        if (liveSessionId == null) {
            throw new ConflictException("The session for this task just ended. Press Run here again.");
        }
        return reuse(liveSessionId, stepId);
    }

    /**
     * Point the task's warm session at another step: release the one it was on, claim the new one,
     * and drop a one-line note into the session so it starts there. No {@code /rk} reload, so the
     * only tokens spent are that note against a cache that is still warm. A {@code null} step, or
     * the one it is already on, just refocuses the session.
     */
    private ClaudeSessionView reuse(UUID sessionId, UUID stepId) {
        Live handle = live.get(sessionId);
        if (handle == null) {
            throw new ConflictException("The session for this task has just ended. Try again.");
        }
        if (stepId != null && !stepId.equals(handle.stepId)) {
            taskSteps.releaseRunning(handle.stepId);
            taskSteps.markRunning(stepId);
            sessions.retargetStep(sessionId, stepId);
            handle.stepId = stepId;
            refreshTaskRunning(handle.taskId);
            String note = taskSteps.titleOf(stepId)
                    .map(title -> "Now on step: " + title)
                    .orElse("Now on another step of this task.");
            emitSystem(sessionId, note);
            writeLine(handle, note);
            sessions.markStatus(sessionId, ClaudeSessionStatus.WORKING).ifPresent(this::emitStatus);
        } else {
            emitSystem(sessionId, "Reusing the session already running for this task.");
        }
        return sessions.find(sessionId)
                .orElseThrow(() -> new ConflictException("The session for this task is gone."));
    }

    /**
     * Drop the conversation so far and reload the task context: {@code /clear} (the CLI eats it,
     * no model tokens) then {@code /rk} on the same process. The system prompt, tool schemas and
     * {@code CLAUDE.md} stay a cache read, so this costs one {@code /rk} load, the same as opening
     * a fresh terminal and typing it.
     */
    public ClaudeSessionView clear(UUID sessionId) {
        Live handle = live.get(sessionId);
        if (handle == null) {
            throw new ConflictException("This session has ended. Start a new one to keep going.");
        }
        ClaudeSessionView view = sessions.find(sessionId)
                .orElseThrow(() -> new ConflictException("This session has ended. Start a new one to keep going."));
        writeLine(handle, "/clear");
        writeLine(handle, "/rk " + view.anchors());
        emitSystem(sessionId, "Context cleared. Reloading " + view.anchors() + " with /rk.");
        sessions.markStatus(sessionId, ClaudeSessionStatus.WORKING).ifPresent(this::emitStatus);
        return sessions.find(sessionId).orElse(view);
    }

    private void emitSystem(UUID sessionId, String text) {
        ClaudeMessageView view = sessions.append(sessionId, ClaudeMessageRole.SYSTEM, text, null, null);
        stream.emit(sessionId, "message", view);
    }

    private UUID liveSessionForTask(UUID taskId) {
        if (taskId == null) {
            return null;
        }
        return live.entrySet().stream()
                .filter(entry -> taskId.equals(entry.getValue().taskId))
                .map(java.util.Map.Entry::getKey)
                .findFirst()
                .orElse(null);
    }

    private ClaudeSessionView spawn(
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
                new BufferedWriter(new OutputStreamWriter(process.getOutputStream(), StandardCharsets.UTF_8)),
                taskId, stepId);
        live.put(id, handle);
        refreshTaskRunning(taskId);
        taskSteps.markRunning(stepId);

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
            refreshTaskRunning(handle.taskId);
            taskSteps.releaseRunning(handle.stepId);
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
        refreshTaskRunning(handle.taskId);
        taskSteps.releaseRunning(handle.stepId);
    }

    /**
     * Tell the task-review line whether a session is still attached to this task
     * with no step target. A stepless task with one sits at {@code RUNNING}; when
     * the last one goes it drops back to {@code OPEN}. Inert for a task that has a
     * checklist, and swallowed on failure because the signal is ambient.
     */
    private void refreshTaskRunning(UUID taskId) {
        if (taskId == null) {
            return;
        }
        boolean anyStepless = live.values().stream()
                .anyMatch(handle -> taskId.equals(handle.taskId) && handle.stepId == null);
        try {
            taskReview.sessionRunning(taskId, anyStepless);
        } catch (RuntimeException ignored) {
            // A missed flip is corrected by the next attach or detach.
        }
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
            if (!handle.process.isAlive()) {
                throw new ConflictException("The session is no longer accepting input.");
            }
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
