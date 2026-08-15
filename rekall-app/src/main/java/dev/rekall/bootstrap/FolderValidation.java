package dev.rekall.bootstrap;

import java.nio.file.Files;
import java.nio.file.Path;

/**
 * What a folder typed into the database-location field turns out to be, before anything commits
 * to it.
 *
 * <p>The folder itself is never created here: a user typing a path that does not exist is told
 * to create it by hand, because silently {@code mkdir}-ing a path they may have mistyped is how
 * a database ends up in the wrong place.
 */
public final class FolderValidation {

    private FolderValidation() {
    }

    public record Result(String resolvedPath, boolean exists, boolean isDirectory, boolean writable, boolean hasDatabase) {

        public boolean usable() {
            return exists && isDirectory && writable;
        }
    }

    public static Result inspect(String rawPath) {
        String trimmed = rawPath == null ? "" : rawPath.trim();
        if (trimmed.isEmpty()) {
            return new Result("", false, false, false, false);
        }
        Path path = expand(trimmed);
        boolean exists = Files.exists(path);
        boolean isDirectory = exists && Files.isDirectory(path);
        boolean writable = isDirectory && Files.isWritable(path);
        boolean hasDatabase = isDirectory && Files.isRegularFile(path.resolve("rekall.mv.db"));
        return new Result(path.toString(), exists, isDirectory, writable, hasDatabase);
    }

    /** A leading {@code ~} is a shell convention the browser never expands, so it is done here. */
    private static Path expand(String trimmed) {
        String withHome = trimmed.equals("~") || trimmed.startsWith("~/")
                ? System.getProperty("user.home") + trimmed.substring(1)
                : trimmed;
        return Path.of(withHome).toAbsolutePath().normalize();
    }
}
