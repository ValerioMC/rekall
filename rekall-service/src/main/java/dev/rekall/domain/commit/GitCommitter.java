package dev.rekall.domain.commit;

import org.springframework.stereotype.Component;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * The one place that writes to a repository: stages everything and commits it. It commits as
 * whoever git is configured to be in that folder; no identity is invented here, so a machine
 * with no {@code user.email} is refused before anything is staged.
 */
@Component
class GitCommitter {

    /** Every change {@code git add -A} would pick up, untracked files listed one by one. */
    List<PendingChange> pendingChanges(Path repoFolder) {
        GitCommand.Result status = GitCommand.run(repoFolder, List.of("status", "--porcelain", "-uall"));
        if (!status.ok()) {
            throw new IllegalArgumentException("Could not read the working tree in " + repoFolder + ": " + status.error());
        }
        List<PendingChange> changes = new ArrayList<>();
        for (String line : status.stdout().split("\n")) {
            if (!line.isBlank()) {
                changes.add(PendingChange.parse(line));
            }
        }
        return changes;
    }

    /** Stages everything, commits it with {@code message}, and hands back the new tip's hash. */
    String commitAll(Path repoFolder, String message) {
        GitCommand.Result staged = GitCommand.run(repoFolder, List.of("add", "-A"));
        if (!staged.ok()) {
            throw new IllegalArgumentException("Could not stage the changes in " + repoFolder + ": " + staged.error());
        }
        GitCommand.Result committed = GitCommand.run(repoFolder, List.of("commit", "-q", "-F", "-"), message);
        if (!committed.ok()) {
            String reason = committed.error().isBlank() ? committed.output() : committed.error();
            throw new IllegalArgumentException("git commit failed in " + repoFolder + ": " + reason);
        }
        GitCommand.Result tip = GitCommand.run(repoFolder, List.of("rev-parse", "HEAD"));
        if (!tip.ok() || tip.output().isBlank()) {
            throw new IllegalArgumentException("Committed, but could not read the new tip in " + repoFolder + ".");
        }
        return tip.output();
    }
}
