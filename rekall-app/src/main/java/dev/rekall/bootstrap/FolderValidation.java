package dev.rekall.bootstrap;

import java.nio.file.Files;
import java.nio.file.Path;

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

    private static Path expand(String trimmed) {
        String withHome = trimmed.equals("~") || trimmed.startsWith("~/")
                ? System.getProperty("user.home") + trimmed.substring(1)
                : trimmed;
        return Path.of(withHome).toAbsolutePath().normalize();
    }
}
