package dev.rekall.claude;

import com.pty4j.PtyProcess;
import com.pty4j.PtyProcessBuilder;
import com.pty4j.WinSize;
import dev.rekall.claude.TerminalApiDtos.TerminalView;
import dev.rekall.common.ConflictException;
import dev.rekall.domain.claude.TerminalLaunchService;
import dev.rekall.domain.claude.TerminalLaunchService.TerminalLaunch;
import dev.rekall.domain.review.TaskReviewService;
import dev.rekall.domain.step.TaskStepService;
import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

/**
 * Owns the {@code claude} pseudo-terminals behind the in-app terminal pane: one PTY per task, each
 * running the interactive {@code claude} TUI. A virtual thread pumps PTY output to every attached
 * {@link Listener} and into a bounded scrollback buffer; {@link #write} and {@link #resize} carry
 * input the other way. The terminal moves the task's checklist marker ({@link TaskStepService}) or
 * review line ({@link TaskReviewService}) as it opens and closes. Nothing is persisted; strays are
 * bounded by a cap, an idle sweep, and a shutdown hook.
 */
@Component
@Slf4j
public class PtyTerminalManager {

    private static final int DEFAULT_COLUMNS = 80;
    private static final int DEFAULT_ROWS = 24;
    private static final int MAX_DIMENSION = 1000;
    private static final int READ_BUFFER = 8192;

    /** Only the aliases the settings offer are accepted; anything else leaves the account default. */
    private static final Set<String> MODEL_ALIASES = Set.of("opus", "sonnet", "haiku", "fable");
    private static final Set<String> EFFORT_LEVELS = Set.of("low", "medium", "high", "xhigh", "max");

    private final TerminalLaunchService launchService;
    private final ClaudeCli cli;
    private final TaskStepService taskSteps;
    private final TaskReviewService taskReview;

    private final int maxLive;
    private final long idleMinutes;
    private final long sweepMinutes;
    private final int scrollbackBytes;

    private final ConcurrentHashMap<UUID, Terminal> live = new ConcurrentHashMap<>();

    /** Serialises the cap check and registration so two fast opens cannot both slip past the cap. */
    private final Object spawnGate = new Object();

    private ScheduledExecutorService reaper;

    public PtyTerminalManager(
            TerminalLaunchService launchService,
            ClaudeCli cli,
            TaskStepService taskSteps,
            TaskReviewService taskReview,
            @Value("${rekall.terminal.max-sessions:8}") int maxLive,
            @Value("${rekall.terminal.idle-minutes:120}") long idleMinutes,
            @Value("${rekall.terminal.sweep-minutes:5}") long sweepMinutes,
            @Value("${rekall.terminal.scrollback-bytes:131072}") int scrollbackBytes) {
        this.launchService = launchService;
        this.cli = cli;
        this.taskSteps = taskSteps;
        this.taskReview = taskReview;
        this.maxLive = maxLive;
        this.idleMinutes = idleMinutes;
        this.sweepMinutes = sweepMinutes;
        this.scrollbackBytes = scrollbackBytes;
    }

    /** A sink for one terminal's output and the moment it ends. Implemented by the socket handler. */
    public interface Listener {
        void output(byte[] data, int length);

        void ended(int exitCode, String detail);
    }

    private final class Terminal {
        private final UUID id;
        private final PtyProcess process;
        private final OutputStream stdin;
        private final TerminalLaunch launch;
        private volatile UUID stepId;
        private final boolean skipPermissions;
        private final String model;
        private final String effort;
        private final Instant startedAt = Instant.now();
        private volatile Instant lastActivityAt = Instant.now();
        private volatile boolean closing;
        private final Object writeLock = new Object();
        private final CopyOnWriteArrayList<Listener> listeners = new CopyOnWriteArrayList<>();
        private final Scrollback scrollback = new Scrollback(scrollbackBytes);

        private Terminal(
                UUID id, PtyProcess process, TerminalLaunch launch, UUID stepId,
                boolean skipPermissions, String model, String effort) {
            this.id = id;
            this.process = process;
            this.stdin = process.getOutputStream();
            this.launch = launch;
            this.stepId = stepId;
            this.skipPermissions = skipPermissions;
            this.model = model;
            this.effort = effort;
        }

        private TerminalView view() {
            return new TerminalView(
                    id, launch.taskId(), stepId, launch.anchors(), launch.workingDir(),
                    launch.projectLabel(), launch.taskLabel(), launch.taskTitle(),
                    skipPermissions, model, effort,
                    process.isAlive() && !closing, startedAt, lastActivityAt);
        }
    }

    // ---------------------------------------------------------------- lifecycle

    @PostConstruct
    void startReaper() {
        reaper = Executors.newSingleThreadScheduledExecutor(runnable -> {
            Thread thread = new Thread(runnable, "terminal-reaper");
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
        List<Terminal> terminals = List.copyOf(live.values());
        live.clear();
        for (Terminal terminal : terminals) {
            terminal.closing = true;
            terminal.process.destroy();
        }
        for (Terminal terminal : terminals) {
            try {
                if (!terminal.process.waitFor(1, TimeUnit.SECONDS)) {
                    terminal.process.destroyForcibly();
                }
            } catch (InterruptedException interrupted) {
                Thread.currentThread().interrupt();
                terminal.process.destroyForcibly();
            }
            notifyEnded(terminal, terminal.process.isAlive() ? 0 : safeExitValue(terminal), "The app was shutting down.");
            releaseRunning(terminal.launch.taskId(), terminal.stepId);
        }
    }

    // ---------------------------------------------------------------- commands

    /**
     * Start a {@code claude} PTY for this task in its project's folder, with {@code /rk} typed first.
     * A missing folder or CLI is a retriable conflict. A second open on a task that already has a
     * terminal is {@linkplain #retarget routed to it}. A non-null {@code stepId} is marked
     * {@code RUNNING} while the terminal is on it; {@code null} means the task itself.
     */
    public TerminalView open(UUID taskId, UUID stepId, boolean skipPermissions, String model, String effort) {
        TerminalLaunch launch = launchService.resolve(taskId);

        Terminal existing = liveTerminalForTask(taskId);
        if (existing != null) {
            return retarget(existing, stepId);
        }

        Path directory = Path.of(launch.workingDir());
        if (!Files.isDirectory(directory)) {
            throw new ConflictException(
                    "This project's folder (" + launch.workingDir() + ") is not there. Set it again on the project page.");
        }

        Path binary = cli.locate().orElseThrow(() -> new ConflictException(
                "Claude Code's command line tool isn't on this machine. Install it, then try again."));

        String chosenModel = normalise(model, MODEL_ALIASES);
        String chosenEffort = normalise(effort, EFFORT_LEVELS);

        List<String> command = new ArrayList<>();
        command.add(binary.toString());
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
        command.add("/rk " + launch.anchors());

        PtyProcessBuilder builder = new PtyProcessBuilder()
                .setCommand(command.toArray(String[]::new))
                .setEnvironment(terminalEnvironment())
                .setDirectory(directory.toString())
                .setInitialColumns(DEFAULT_COLUMNS)
                .setInitialRows(DEFAULT_ROWS)
                .setConsole(false);

        UUID id = UUID.randomUUID();
        Terminal terminal;
        synchronized (spawnGate) {
            if (live.size() >= maxLive) {
                throw new ConflictException(
                        "Rekall is already running " + maxLive + " terminals. Close one before opening another.");
            }
            PtyProcess process;
            try {
                process = builder.start();
            } catch (IOException startFailed) {
                throw new ConflictException("Could not start claude: " + startFailed.getMessage());
            }
            terminal = new Terminal(id, process, launch, stepId, skipPermissions, chosenModel, chosenEffort);
            live.put(id, terminal);
        }

        markRunning(taskId, stepId);
        Thread.ofVirtual().name("terminal-pty-" + id).start(() -> pump(terminal));
        terminal.process.onExit().thenAccept(exited -> onExit(id));

        log.info("Opened terminal {} on {} in {}", id, launch.anchors(), launch.workingDir());
        return terminal.view();
    }

    /** Move the running marker to another step without touching the process. Null or unchanged is a no-op. */
    private TerminalView retarget(Terminal terminal, UUID stepId) {
        if (stepId != null && !stepId.equals(terminal.stepId)) {
            releaseRunning(terminal.launch.taskId(), terminal.stepId);
            terminal.stepId = stepId;
            markRunning(terminal.launch.taskId(), stepId);
        }
        terminal.lastActivityAt = Instant.now();
        return terminal.view();
    }

    private void markRunning(UUID taskId, UUID stepId) {
        if (stepId != null) {
            taskSteps.markRunning(stepId);
        } else {
            taskReview.sessionRunning(taskId, true);
        }
    }

    private void releaseRunning(UUID taskId, UUID stepId) {
        if (stepId != null) {
            taskSteps.releaseRunning(stepId);
        } else {
            taskReview.sessionRunning(taskId, false);
        }
    }

    /** Attach a pane: replay the scrollback, then follow live output. A closed terminal is a conflict. */
    public TerminalView attach(UUID id, Listener listener) {
        Terminal terminal = require(id);
        // Subscribe before replaying: a chunk arriving in the gap is drawn twice, not dropped.
        terminal.listeners.add(listener);
        byte[] backlog = terminal.scrollback.snapshot();
        if (backlog.length > 0) {
            listener.output(backlog, backlog.length);
        }
        return terminal.view();
    }

    public void detach(UUID id, Listener listener) {
        Terminal terminal = live.get(id);
        if (terminal != null) {
            terminal.listeners.remove(listener);
        }
    }

    /** Feed raw bytes to the terminal's stdin. Serialised per terminal; a dead PTY is a conflict. */
    public void write(UUID id, byte[] data) {
        Terminal terminal = require(id);
        synchronized (terminal.writeLock) {
            if (!terminal.process.isAlive()) {
                throw new ConflictException("This terminal has closed.");
            }
            try {
                terminal.stdin.write(data);
                terminal.stdin.flush();
            } catch (IOException refused) {
                throw new ConflictException("This terminal is no longer accepting input.");
            }
        }
        terminal.lastActivityAt = Instant.now();
    }

    /** Tell the PTY its new window size. Out-of-range values are clamped; a failure is logged, not thrown. */
    public void resize(UUID id, int columns, int rows) {
        Terminal terminal = live.get(id);
        if (terminal == null) {
            return;
        }
        int safeColumns = Math.clamp(columns, 1, MAX_DIMENSION);
        int safeRows = Math.clamp(rows, 1, MAX_DIMENSION);
        try {
            terminal.process.setWinSize(new WinSize(safeColumns, safeRows));
        } catch (RuntimeException resizeFailed) {
            log.debug("Resize of terminal {} to {}x{} failed: {}", id, safeColumns, safeRows, resizeFailed.getMessage());
        }
    }

    /** Stop a terminal now: SIGTERM, a short grace, then SIGKILL. Idempotent. */
    public TerminalView close(UUID id, String reason) {
        Terminal terminal = live.remove(id);
        if (terminal == null) {
            return null;
        }
        terminal.closing = true;
        terminal.process.destroy();
        try {
            if (!terminal.process.waitFor(2, TimeUnit.SECONDS)) {
                terminal.process.destroyForcibly();
            }
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            terminal.process.destroyForcibly();
        }
        notifyEnded(terminal, safeExitValue(terminal), reason);
        releaseRunning(terminal.launch.taskId(), terminal.stepId);
        log.info("Closed terminal {} ({})", id, reason);
        return terminal.view();
    }

    public List<TerminalView> list() {
        return live.values().stream().map(Terminal::view).toList();
    }

    public Optional<TerminalView> get(UUID id) {
        return Optional.ofNullable(live.get(id)).map(Terminal::view);
    }

    public boolean isLive(UUID id) {
        Terminal terminal = live.get(id);
        return terminal != null && terminal.process.isAlive();
    }

    public int liveCount() {
        return live.size();
    }

    // ---------------------------------------------------------------- pty io

    private void pump(Terminal terminal) {
        byte[] buffer = new byte[READ_BUFFER];
        try (InputStream out = terminal.process.getInputStream()) {
            int read;
            while ((read = out.read(buffer)) != -1) {
                if (read == 0) {
                    continue;
                }
                terminal.scrollback.append(buffer, read);
                for (Listener listener : terminal.listeners) {
                    try {
                        listener.output(buffer, read);
                    } catch (RuntimeException listenerFailed) {
                        log.debug("Dropping a listener on terminal {}: {}", terminal.id, listenerFailed.getMessage());
                        terminal.listeners.remove(listener);
                    }
                }
            }
        } catch (IOException ended) {
            log.debug("PTY output for terminal {} closed: {}", terminal.id, ended.getMessage());
        }
    }

    private void onExit(UUID id) {
        Terminal terminal = live.remove(id);
        if (terminal == null) {
            return;
        }
        int code = safeExitValue(terminal);
        notifyEnded(terminal, code, code == 0 ? "The terminal ended." : "claude exited with code " + code);
        releaseRunning(terminal.launch.taskId(), terminal.stepId);
        log.info("Terminal {} exited with code {}", id, code);
    }

    private void notifyEnded(Terminal terminal, int exitCode, String detail) {
        for (Listener listener : terminal.listeners) {
            try {
                listener.ended(exitCode, detail);
            } catch (RuntimeException ignored) {
                // The socket is already gone.
            }
        }
        terminal.listeners.clear();
    }

    private void sweepIdle() {
        try {
            Instant cutoff = Instant.now().minus(Duration.ofMinutes(idleMinutes));
            for (Terminal terminal : List.copyOf(live.values())) {
                if (terminal.lastActivityAt.isBefore(cutoff)) {
                    close(terminal.id, "Closed after " + idleMinutes + " minutes with no activity.");
                }
            }
        } catch (RuntimeException sweepFailed) {
            log.warn("Terminal idle sweep failed: {}", sweepFailed.getMessage());
        }
    }

    // ---------------------------------------------------------------- helpers

    private Terminal require(UUID id) {
        Terminal terminal = live.get(id);
        if (terminal == null) {
            throw new ConflictException("This terminal has closed. Open a new one to keep going.");
        }
        return terminal;
    }

    private Terminal liveTerminalForTask(UUID taskId) {
        return live.values().stream()
                .filter(terminal -> taskId.equals(terminal.launch.taskId()))
                .findFirst()
                .orElse(null);
    }

    /** {@link ClaudeCli#environment()} plus the {@code TERM}/{@code COLORTERM}/locale a TUI needs. */
    private Map<String, String> terminalEnvironment() {
        Map<String, String> environment = new HashMap<>(cli.environment());
        environment.put("TERM", "xterm-256color");
        environment.put("COLORTERM", "truecolor");
        for (String key : List.of("LANG", "LC_ALL", "LC_CTYPE")) {
            String value = System.getenv(key);
            if (value != null && !value.isBlank()) {
                environment.put(key, value);
            }
        }
        return environment;
    }

    private static int safeExitValue(Terminal terminal) {
        try {
            return terminal.process.exitValue();
        } catch (IllegalThreadStateException stillRunning) {
            return -1;
        }
    }

    private static String normalise(String value, Set<String> allowed) {
        if (value == null) {
            return null;
        }
        String trimmed = value.strip().toLowerCase(Locale.ROOT);
        return allowed.contains(trimmed) ? trimmed : null;
    }

    /** A fixed-size ring of recent PTY bytes so a late-opening pane repaints. A snapshot may start
        mid escape-sequence and flicker once on attach. */
    private static final class Scrollback {
        private final byte[] ring;
        private int size;
        private int start;

        private Scrollback(int capacity) {
            this.ring = new byte[Math.max(1, capacity)];
        }

        synchronized void append(byte[] data, int length) {
            for (int index = 0; index < length; index++) {
                ring[(start + size) % ring.length] = data[index];
                if (size < ring.length) {
                    size++;
                } else {
                    start = (start + 1) % ring.length;
                }
            }
        }

        synchronized byte[] snapshot() {
            byte[] out = new byte[size];
            for (int index = 0; index < size; index++) {
                out[index] = ring[(start + index) % ring.length];
            }
            return out;
        }
    }
}
