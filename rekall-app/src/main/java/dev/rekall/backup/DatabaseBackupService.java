package dev.rekall.backup;

import dev.rekall.common.ConflictException;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.context.event.ApplicationReadyEvent;
import org.springframework.context.event.ContextClosedEvent;
import org.springframework.context.event.EventListener;
import org.springframework.dao.DataAccessException;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.stereotype.Service;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Stream;

/**
 * Keeps rotating copies of the open database in a {@code backups} folder next to it, written with
 * H2's {@code BACKUP TO}, which copies the file online and consistently while the application
 * keeps running.
 *
 * <ul>
 *   <li>A backup is taken when the application is ready and then checked every hour: one is due
 *       when the newest backup of any kind is older than {@code rekall.backup.interval-hours}.</li>
 *   <li>{@link #backUp} takes one on demand, and a restore takes one first.</li>
 *   <li>Only the newest {@code rekall.backup.keep} are kept, whatever their reason.</li>
 *   <li>A database with no file (in memory) has no backups; nothing here runs on one.</li>
 * </ul>
 *
 * <p>A failure is logged and reported by {@link #status()}; it never stops the application.
 */
@Slf4j
@Service
public class DatabaseBackupService {

    /** The only names this folder hands out or accepts back, which is what keeps a name from being a path. */
    static final Pattern NAME = Pattern.compile(
            "^rekall-(\\d{8}-\\d{6})(?:-(\\d{1,3}))?-(auto|manual|before-restore|uploaded)\\.zip$");

    private static final DateTimeFormatter STAMP = DateTimeFormatter.ofPattern("yyyyMMdd-HHmmss");

    private static final Duration CHECK_EVERY = Duration.ofHours(1);

    private static final Duration FIRST_CHECK_AFTER = Duration.ofSeconds(5);

    private final JdbcTemplate jdbc;
    private final Optional<DatabaseLocation> location;
    private final boolean enabled;
    private final Duration interval;
    private final int keep;
    private final Clock clock;

    private ScheduledExecutorService scheduler;
    private volatile String lastFailure;

    @Autowired
    public DatabaseBackupService(
            JdbcTemplate jdbc,
            @Value("${spring.datasource.url}") String jdbcUrl,
            @Value("${rekall.backup.enabled:true}") boolean enabled,
            @Value("${rekall.backup.interval-hours:24}") long intervalHours,
            @Value("${rekall.backup.keep:10}") int keep) {
        this(jdbc, DatabaseLocation.of(jdbcUrl), enabled, Duration.ofHours(intervalHours), keep, Clock.systemUTC());
    }

    DatabaseBackupService(JdbcTemplate jdbc, Optional<DatabaseLocation> location, boolean enabled,
                          Duration interval, int keep, Clock clock) {
        this.jdbc = jdbc;
        this.location = location;
        this.enabled = enabled;
        this.interval = interval;
        this.keep = Math.max(1, keep);
        this.clock = clock;
    }

    /** What the settings panel shows: where backups go, which there are, and the last thing that failed. */
    public record Status(boolean available, String folder, long intervalHours, int keep,
                         List<BackupFile> backups, String lastFailure) {
    }

    public Status status() {
        return new Status(
                location.isPresent(),
                location.map(found -> found.backups().toString()).orElse(null),
                interval.toHours(),
                keep,
                location.isPresent() ? list() : List.of(),
                lastFailure);
    }

    public Optional<DatabaseLocation> location() {
        return location;
    }

    /** Takes a backup now and prunes the oldest past {@code keep}. */
    public synchronized BackupFile backUp(BackupFile.Reason reason) {
        DatabaseLocation found = location.orElseThrow(() ->
                new ConflictException("This database is not a file on disk, so it has no backups."));
        try {
            Files.createDirectories(found.backups());
            Path target = freshName(found.backups(), reason);
            jdbc.execute("BACKUP TO '" + target.toString().replace("'", "''") + "'");
            prune(found.backups());
            lastFailure = null;
            log.info("Backed up the database to {}", target);
            return describe(target).orElseThrow();
        } catch (IOException e) {
            throw failed(new UncheckedIOException("Could not write to " + found.backups(), e));
        } catch (DataAccessException e) {
            throw failed(new IllegalStateException("H2 refused the backup: " + e.getMostSpecificCause().getMessage(), e));
        }
    }

    /** Newest first. */
    public List<BackupFile> list() {
        Path folder = location.map(DatabaseLocation::backups).orElse(null);
        if (folder == null || !Files.isDirectory(folder)) {
            return List.of();
        }
        try (Stream<Path> files = Files.list(folder)) {
            return files.map(DatabaseBackupService::describe)
                    .flatMap(Optional::stream)
                    .sorted(Comparator.comparing(BackupFile::name).reversed())
                    .toList();
        } catch (IOException e) {
            throw new UncheckedIOException("Could not read " + folder, e);
        }
    }

    /** The file behind a name this service handed out; anything else is refused. */
    public Path resolve(String name) {
        if (name == null || !NAME.matcher(name).matches()) {
            throw new IllegalArgumentException("'" + name + "' is not the name of a backup");
        }
        DatabaseLocation found = location.orElseThrow(() ->
                new ConflictException("This database is not a file on disk, so it has no backups."));
        Path file = found.backups().resolve(name);
        if (!Files.isRegularFile(file)) {
            throw new IllegalArgumentException("There is no backup called '" + name + "'");
        }
        return file;
    }

    /** A name for a file about to be written into the backups folder, with the reason in it. */
    Path freshName(Path folder, BackupFile.Reason reason) {
        String stamp = LocalDateTime.ofInstant(Instant.now(clock), ZoneOffset.UTC).format(STAMP);
        Path candidate = folder.resolve("rekall-%s-%s.zip".formatted(stamp, reason.slug()));
        for (int attempt = 1; Files.exists(candidate); attempt++) {
            candidate = folder.resolve("rekall-%s-%d-%s.zip".formatted(stamp, attempt, reason.slug()));
        }
        return candidate;
    }

    void backUpIfDue() {
        if (!enabled || location.isEmpty()) {
            return;
        }
        Instant cutoff = Instant.now(clock).minus(interval);
        boolean due = list().stream().findFirst().map(newest -> newest.createdAt().isBefore(cutoff)).orElse(true);
        if (!due) {
            return;
        }
        try {
            backUp(BackupFile.Reason.AUTO);
        } catch (RuntimeException e) {
            log.warn("The scheduled backup failed: {}", e.getMessage());
        }
    }

    @EventListener(ApplicationReadyEvent.class)
    public void startSchedule() {
        if (!enabled || location.isEmpty()) {
            return;
        }
        scheduler = Executors.newSingleThreadScheduledExecutor(runnable -> {
            Thread thread = new Thread(runnable, "rekall-backup");
            thread.setDaemon(true);
            return thread;
        });
        scheduler.scheduleWithFixedDelay(this::backUpIfDue,
                FIRST_CHECK_AFTER.toSeconds(), CHECK_EVERY.toSeconds(), TimeUnit.SECONDS);
    }

    @EventListener(ContextClosedEvent.class)
    public void stopSchedule() {
        if (scheduler != null) {
            scheduler.shutdownNow();
            scheduler = null;
        }
    }

    private void prune(Path folder) {
        List<BackupFile> all = list();
        for (BackupFile old : all.subList(Math.min(keep, all.size()), all.size())) {
            try {
                Files.deleteIfExists(folder.resolve(old.name()));
            } catch (IOException e) {
                log.warn("Could not delete the old backup {}: {}", old.name(), e.getMessage());
            }
        }
    }

    private RuntimeException failed(RuntimeException failure) {
        lastFailure = failure.getMessage();
        log.warn("Backup failed: {}", failure.getMessage());
        return failure;
    }

    static Optional<BackupFile> describe(Path file) {
        Matcher name = NAME.matcher(file.getFileName().toString());
        if (!name.matches() || !Files.isRegularFile(file)) {
            return Optional.empty();
        }
        try {
            Instant created = LocalDateTime.parse(name.group(1), STAMP).toInstant(ZoneOffset.UTC);
            return Optional.of(new BackupFile(
                    file.getFileName().toString(), Files.size(file), created, BackupFile.Reason.ofSlug(name.group(3))));
        } catch (IOException e) {
            log.warn("Skipping the backup {}, which could not be read: {}", file, e.getMessage());
            return Optional.empty();
        }
    }
}
