package dev.rekall.domain.commit;

import org.springframework.stereotype.Component;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.time.OffsetDateTime;
import java.time.format.DateTimeParseException;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;
import java.util.regex.Pattern;

/**
 * Reads commits from a repository on disk: the tip, one named by its hash, or the recent log.
 * Never writes; a bad folder or an unknown hash is reported, not thrown as an I/O crash.
 */
@Component
class GitLogReader {

    /** ASCII unit separator: not going to show up in a commit subject by accident. */
    private static final char FIELD_SEPARATOR = (char) 0x1F;

    /** Abbreviated or full: what a person pastes from {@code git log}. Anything else is not a hash. */
    private static final Pattern HASH = Pattern.compile("[0-9a-fA-F]{4,40}");

    /** A commit read with its patch, ready to be logged against a task. */
    record Commit(String hash, String subject, String diff) {
    }

    /** One line of the recent log: enough to recognise a commit, without its patch. */
    record LogEntry(String hash, String subject, Instant committedAt) {
    }

    Commit head(Path repoFolder) {
        return commit(repoFolder, "HEAD");
    }

    /**
     * The commit a hash names. The hash is validated as hex before it ever reaches git, so a
     * pasted string cannot become a flag; a prefix that matches no commit, or more than one, is
     * refused with git's own reason.
     */
    Commit commit(Path repoFolder, String ref) {
        requireRepository(repoFolder);
        if (!"HEAD".equals(ref) && !HASH.matcher(ref).matches()) {
            throw new IllegalArgumentException(
                    "'%s' is not a commit hash: paste 4 to 40 hex characters from git log.".formatted(ref));
        }
        boolean tip = "HEAD".equals(ref);
        String output = run(repoFolder, List.of("log", "-1", "--format=%H" + FIELD_SEPARATOR + "%s", ref, "--"),
                tip ? "Could not read the last commit in " + repoFolder : "No commit '%s' in %s.".formatted(ref, repoFolder),
                tip);

        int separator = output.indexOf(FIELD_SEPARATOR);
        if (separator < 0) {
            throw new IllegalArgumentException("Unexpected git output in " + repoFolder + ".");
        }
        String hash = output.substring(0, separator);
        return new Commit(hash, output.substring(separator + 1), diffOf(repoFolder, hash));
    }

    /** The newest {@code limit} commits, newest first: what the console lists when picking one by hand. */
    List<LogEntry> recent(Path repoFolder, int limit) {
        requireRepository(repoFolder);
        String format = "%H" + FIELD_SEPARATOR + "%s" + FIELD_SEPARATOR + "%cI";
        String output = run(repoFolder, List.of("log", "-n", Integer.toString(limit), "--format=" + format, "--"),
                "Could not read the log in " + repoFolder, true);

        List<LogEntry> entries = new ArrayList<>();
        for (String line : output.split("\n")) {
            String[] fields = line.split(String.valueOf(FIELD_SEPARATOR), -1);
            if (fields.length != 3) {
                throw new IllegalArgumentException("Unexpected git output in " + repoFolder + ".");
            }
            entries.add(new LogEntry(fields[0], fields[1], parseInstant(fields[2])));
        }
        return entries;
    }

    private static void requireRepository(Path repoFolder) {
        if (!Files.isDirectory(repoFolder)) {
            throw new IllegalArgumentException("This project's folder (" + repoFolder + ") is not there.");
        }
    }

    /** {@code %cI} is strict ISO 8601 with the committer's offset, which {@link Instant#parse} would refuse. */
    private static Instant parseInstant(String iso) {
        try {
            return OffsetDateTime.parse(iso).toInstant();
        } catch (DateTimeParseException malformed) {
            throw new IllegalArgumentException("Unexpected commit date from git: " + iso);
        }
    }

    /**
     * Runs one read-only git command and hands back its stdout. A non-zero exit becomes
     * {@code failure}, with git's own stderr appended when {@code withReason} is set: useful for
     * "no commits yet", noise for "bad revision", where the hash in the message already says it all.
     */
    private static String run(Path repoFolder, List<String> arguments, String failure, boolean withReason) {
        List<String> command = new ArrayList<>(List.of("git", "-C", repoFolder.toString()));
        command.addAll(arguments);

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
            if (!withReason) {
                throw new IllegalArgumentException(failure);
            }
            String detail = error.isBlank() ? "there is no commit yet" : error;
            throw new IllegalArgumentException(failure + ": " + detail);
        }
        return output;
    }

    /**
     * The patch this commit introduced, so it can be reread later without a checkout. Best effort:
     * a diff that cannot be produced (a huge merge, an odd git state) is dropped rather than
     * failing the commit that {@link #commit} otherwise read cleanly.
     */
    private String diffOf(Path repoFolder, String hash) {
        List<String> command = List.of("git", "-C", repoFolder.toString(), "show", "--format=", hash, "--");
        try {
            Process process = new ProcessBuilder(command).start();
            String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
            process.getErrorStream().readAllBytes();
            if (!process.waitFor(5, TimeUnit.SECONDS)) {
                process.destroyForcibly();
                return null;
            }
            return process.exitValue() == 0 ? output.strip() : null;
        } catch (IOException failed) {
            return null;
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            return null;
        }
    }
}
