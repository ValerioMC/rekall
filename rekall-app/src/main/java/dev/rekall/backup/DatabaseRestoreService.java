package dev.rekall.backup;

import dev.rekall.common.ConflictException;
import dev.rekall.ApplicationRestarter;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.Arrays;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

/**
 * Puts a backup back as the database, which is also how a database moves to another folder or
 * another machine: download a backup there, restore it here.
 *
 * <p>A restore replaces everything, so nothing is touched until three things hold: the zip holds
 * an H2 database file, a backup of what is there now has been taken (so the restore can itself be
 * undone), and the file has been extracted beside the live one. Only then does the application
 * restart, and in the gap between the old context closing and the new one starting the database
 * is shut down and the file swapped in. The new context runs the Liquibase changelogs as always,
 * so a backup from an older version comes back migrated.
 */
@Slf4j
@Service
public class DatabaseRestoreService {

    /** The first bytes of every file H2's MVStore writes. */
    private static final byte[] MVSTORE_HEADER = "H:2".getBytes(StandardCharsets.US_ASCII);

    private static final String STAGED_SUFFIX = ".restoring";

    private final DatabaseBackupService backups;
    private final String jdbcUrl;
    private final String username;
    private final String password;

    public DatabaseRestoreService(
            DatabaseBackupService backups,
            @Value("${spring.datasource.url}") String jdbcUrl,
            @Value("${spring.datasource.username:}") String username,
            @Value("${spring.datasource.password:}") String password) {
        this.backups = backups;
        this.jdbcUrl = jdbcUrl;
        this.username = username;
        this.password = password;
    }

    /** Restores one of the backups in the folder, by the name it is listed under. */
    public BackupFile restore(String name) {
        return restoreFrom(backups.resolve(name));
    }

    /** Keeps an uploaded backup in the folder as {@code …-uploaded.zip}, then restores it. */
    public BackupFile restoreUpload(InputStream upload) {
        DatabaseLocation location = requireLocation();
        Path kept;
        try {
            Files.createDirectories(location.backups());
            kept = backups.freshName(location.backups(), BackupFile.Reason.UPLOADED);
            Files.copy(upload, kept);
        } catch (IOException e) {
            throw new UncheckedIOException("Could not keep the uploaded backup", e);
        }
        try {
            return restoreFrom(kept);
        } catch (RuntimeException refused) {
            deleteQuietly(kept);
            throw refused;
        }
    }

    private BackupFile restoreFrom(Path zip) {
        DatabaseLocation location = requireLocation();
        Path staged = location.folder().resolve(location.dataFileName() + STAGED_SUFFIX);
        extractDatabase(zip, location.dataFileName(), staged);
        if (!ApplicationRestarter.canRestart()) {
            deleteQuietly(staged);
            throw new ConflictException("This instance cannot restart itself, so nothing was restored.");
        }

        BackupFile safety = backups.backUp(BackupFile.Reason.BEFORE_RESTORE);
        boolean scheduled = ApplicationRestarter.restart(() -> { }, () -> swap(staged, location.dataFile()));
        if (!scheduled) {
            deleteQuietly(staged);
            throw new ConflictException("This instance cannot restart itself, so nothing was restored.");
        }
        log.info("Restoring {} over {}; the previous state is in {}", zip.getFileName(), location.dataFile(), safety.name());
        return safety;
    }

    /**
     * Writes the database file out of the zip, refusing a zip that does not hold one. A file with
     * the database's own name wins; otherwise the zip has to hold exactly one {@code .mv.db}, which
     * is what lets a backup taken from a database with another base name be restored here.
     */
    static void extractDatabase(Path zip, String dataFileName, Path target) {
        String entryName = databaseEntryIn(zip, dataFileName);
        try (ZipInputStream in = new ZipInputStream(Files.newInputStream(zip))) {
            for (ZipEntry entry = in.getNextEntry(); entry != null; entry = in.getNextEntry()) {
                if (!entry.getName().equals(entryName)) {
                    continue;
                }
                try (OutputStream out = Files.newOutputStream(target)) {
                    in.transferTo(out);
                }
                if (!startsWithMvStoreHeader(target)) {
                    deleteQuietly(target);
                    throw new IllegalArgumentException(
                            "'" + entryName + "' in that zip is not an H2 database file. Nothing was restored.");
                }
                return;
            }
        } catch (IOException e) {
            deleteQuietly(target);
            throw new IllegalArgumentException("That file is not a zip Rekall can read. Nothing was restored.", e);
        }
    }

    private static String databaseEntryIn(Path zip, String dataFileName) {
        String single = null;
        int databases = 0;
        try (ZipInputStream in = new ZipInputStream(Files.newInputStream(zip))) {
            for (ZipEntry entry = in.getNextEntry(); entry != null; entry = in.getNextEntry()) {
                String name = entry.getName();
                if (entry.isDirectory() || name.contains("/") || name.contains("\\") || !name.endsWith(".mv.db")) {
                    continue;
                }
                if (name.equals(dataFileName)) {
                    return name;
                }
                databases++;
                single = name;
            }
        } catch (IOException e) {
            throw new IllegalArgumentException("That file is not a zip Rekall can read. Nothing was restored.", e);
        }
        if (databases != 1) {
            throw new IllegalArgumentException(databases == 0
                    ? "That zip holds no H2 database file (*.mv.db). Nothing was restored."
                    : "That zip holds more than one database file. Nothing was restored.");
        }
        return single;
    }

    private static boolean startsWithMvStoreHeader(Path file) throws IOException {
        try (InputStream in = Files.newInputStream(file)) {
            byte[] head = in.readNBytes(MVSTORE_HEADER.length);
            return Arrays.equals(head, MVSTORE_HEADER);
        }
    }

    /**
     * Runs with no context up: shuts H2 down (DB_CLOSE_DELAY=-1 keeps it open past the pool) and
     * swaps the staged file in. If the swap fails the old file is still in place and the
     * application comes back on it, with the failure in the log.
     */
    private void swap(Path staged, Path live) {
        try (Connection connection = DriverManager.getConnection(jdbcUrl, username, password);
             Statement statement = connection.createStatement()) {
            statement.execute("SHUTDOWN");
        } catch (SQLException e) {
            log.warn("Could not shut the database down before the restore: {}", e.getMessage());
        }
        try {
            Files.move(staged, live, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
            log.info("Restored the database file {}", live);
        } catch (IOException e) {
            log.error("The restore could not replace {}; the application restarts on the file it had", live, e);
            deleteQuietly(staged);
        }
    }

    private DatabaseLocation requireLocation() {
        return backups.location().orElseThrow(() ->
                new ConflictException("This database is not a file on disk, so it cannot be restored."));
    }

    private static void deleteQuietly(Path file) {
        try {
            Files.deleteIfExists(file);
        } catch (IOException e) {
            log.warn("Could not delete {}: {}", file, e.getMessage());
        }
    }
}
