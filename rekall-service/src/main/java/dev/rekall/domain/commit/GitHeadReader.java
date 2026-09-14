package dev.rekall.domain.commit;

import org.springframework.stereotype.Component;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.TimeUnit;

/** Reads the tip commit of a repository on disk. Never writes; a bad folder is reported, not thrown as an I/O crash. */
@Component
class GitHeadReader {

    /** ASCII unit separator: not going to show up in a commit subject by accident. */
    private static final char FIELD_SEPARATOR = (char) 0x1F;

    record Commit(String hash, String subject) {
    }

    Commit head(Path repoFolder) {
        if (!Files.isDirectory(repoFolder)) {
            throw new IllegalArgumentException("This project's folder (" + repoFolder + ") is not there.");
        }
        List<String> command = List.of(
                "git", "-C", repoFolder.toString(), "log", "-1", "--format=%H" + FIELD_SEPARATOR + "%s");

        Process process;
        try {
            process = new ProcessBuilder(command).start();
        } catch (IOException failed) {
            throw new IllegalArgumentException("Could not run git in " + repoFolder + ": " + failed.getMessage());
        }

        String output;
        String error;
        try {
            output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8).strip();
            error = new String(process.getErrorStream().readAllBytes(), StandardCharsets.UTF_8).strip();
            if (!process.waitFor(5, TimeUnit.SECONDS)) {
                process.destroyForcibly();
                throw new IllegalArgumentException("git took too long to answer in " + repoFolder + ".");
            }
        } catch (IOException failed) {
            throw new IllegalArgumentException("Could not read git's output in " + repoFolder + ": " + failed.getMessage());
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            throw new IllegalArgumentException("Interrupted while reading git's output in " + repoFolder + ".");
        }

        if (process.exitValue() != 0 || output.isBlank()) {
            String detail = error.isBlank() ? "there is no commit yet" : error;
            throw new IllegalArgumentException("Could not read the last commit in " + repoFolder + ": " + detail);
        }

        int separator = output.indexOf(FIELD_SEPARATOR);
        if (separator < 0) {
            throw new IllegalArgumentException("Unexpected git output in " + repoFolder + ".");
        }
        return new Commit(output.substring(0, separator), output.substring(separator + 1));
    }
}
