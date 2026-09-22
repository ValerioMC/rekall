package dev.rekall.backup;

import java.nio.file.Path;
import java.util.Optional;

/**
 * Where the open database lives on disk, read from its JDBC URL: the folder that holds it and the
 * base name its files share ({@code rekall} for {@code rekall.mv.db}). Only a file database has
 * one; an in-memory database, which is what the tests and the first-run setup run on, has none,
 * and backups are then simply not available.
 */
public record DatabaseLocation(Path folder, String baseName) {

    private static final String FILE_PREFIX = "jdbc:h2:file:";

    public static Optional<DatabaseLocation> of(String jdbcUrl) {
        if (jdbcUrl == null || !jdbcUrl.startsWith(FILE_PREFIX)) {
            return Optional.empty();
        }
        String rest = jdbcUrl.substring(FILE_PREFIX.length());
        int options = rest.indexOf(';');
        Path base = Path.of(options < 0 ? rest : rest.substring(0, options)).toAbsolutePath().normalize();
        if (base.getParent() == null || base.getFileName() == null) {
            return Optional.empty();
        }
        return Optional.of(new DatabaseLocation(base.getParent(), base.getFileName().toString()));
    }

    public Path backups() {
        return folder.resolve("backups");
    }

    /** The one file H2's MVStore keeps a database in. */
    public Path dataFile() {
        return folder.resolve(dataFileName());
    }

    public String dataFileName() {
        return baseName + ".mv.db";
    }
}
