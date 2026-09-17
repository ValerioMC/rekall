package dev.rekall.domain.commit;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;

/**
 * One {@code git} invocation in a folder, with its exit code and both streams captured. The
 * callers decide what a failure means; this only makes sure a hung or unrunnable git is reported
 * rather than waited on forever.
 */
final class GitCommand {

    private static final long TIMEOUT_SECONDS = 15;

    /**
     * What git answered. {@code stdout} is untouched, since a porcelain status line starts with
     * a meaningful space; {@link #output()} and {@link #error()} are the trimmed reads.
     */
    record Result(int exitCode, String stdout, String stderr) {

        boolean ok() {
            return exitCode == 0;
        }

        String output() {
            return stdout.strip();
        }

        String error() {
            return stderr.strip();
        }
    }

    private GitCommand() {
    }

    static Result run(Path repoFolder, List<String> arguments) {
        return run(repoFolder, arguments, null);
    }

    /** Runs git with {@code stdin} fed to it when not null: how a multi-line commit message travels. */
    static Result run(Path repoFolder, List<String> arguments, String stdin) {
        List<String> command = new ArrayList<>(List.of("git", "-C", repoFolder.toString()));
        command.addAll(arguments);

        Process process;
        try {
            process = new ProcessBuilder(command).start();
        } catch (IOException failed) {
            throw new IllegalArgumentException("Could not run git in " + repoFolder + ": " + failed.getMessage());
        }

        try {
            try (OutputStream in = process.getOutputStream()) {
                if (stdin != null) {
                    in.write(stdin.getBytes(StandardCharsets.UTF_8));
                }
            }
            String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
            String error = new String(process.getErrorStream().readAllBytes(), StandardCharsets.UTF_8);
            if (!process.waitFor(TIMEOUT_SECONDS, TimeUnit.SECONDS)) {
                process.destroyForcibly();
                throw new IllegalArgumentException("git took too long to answer in " + repoFolder + ".");
            }
            return new Result(process.exitValue(), output, error);
        } catch (IOException failed) {
            throw new IllegalArgumentException("Could not read git's output in " + repoFolder + ": " + failed.getMessage());
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            throw new IllegalArgumentException("Interrupted while reading git's output in " + repoFolder + ".");
        }
    }
}
