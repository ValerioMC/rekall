package dev.rekall.domain.commit;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

/** Exercises the real {@code git} binary: this class is the write half of the process wrapper. */
class GitCommitterTest {

    private final GitCommitter committer = new GitCommitter();

    @Test
    @DisplayName("lists untracked, modified and deleted files as pending changes")
    void listsPendingChanges(@TempDir Path repo) throws Exception {
        initWithIdentity(repo);
        Files.writeString(repo.resolve("kept.txt"), "one");
        Files.writeString(repo.resolve("gone.txt"), "two");
        run(repo, "add", "-A");
        run(repo, "commit", "-q", "-m", "seed");
        Files.writeString(repo.resolve("kept.txt"), "one more");
        Files.delete(repo.resolve("gone.txt"));
        Files.createDirectories(repo.resolve("src"));
        Files.writeString(repo.resolve("src/new.txt"), "three");

        List<PendingChange> changes = committer.pendingChanges(repo);

        assertThat(changes).containsExactlyInAnyOrder(
                new PendingChange(PendingChange.Kind.MODIFIED, "kept.txt"),
                new PendingChange(PendingChange.Kind.DELETED, "gone.txt"),
                new PendingChange(PendingChange.Kind.ADDED, "src/new.txt"));
    }

    @Test
    @DisplayName("a clean tree has no pending change")
    void aCleanTreeHasNoPendingChange(@TempDir Path repo) throws Exception {
        initWithIdentity(repo);
        Files.writeString(repo.resolve("a.txt"), "a");
        run(repo, "add", "-A");
        run(repo, "commit", "-q", "-m", "seed");

        assertThat(committer.pendingChanges(repo)).isEmpty();
    }

    @Test
    @DisplayName("commits everything with the multi-line message and returns the new tip")
    void commitsEverythingWithTheMessage(@TempDir Path repo) throws Exception {
        initWithIdentity(repo);
        Files.writeString(repo.resolve("a.txt"), "a");
        Files.writeString(repo.resolve("b.txt"), "b");

        String hash = committer.commitAll(repo, "feat: the subject\n\nproject:vega task:x\n2 files: 2 added\n");

        assertThat(hash).isEqualTo(run(repo, "rev-parse", "HEAD"));
        assertThat(run(repo, "log", "-1", "--format=%s")).isEqualTo("feat: the subject");
        assertThat(run(repo, "log", "-1", "--format=%b")).contains("project:vega task:x");
        assertThat(run(repo, "status", "--porcelain")).isEmpty();
    }

    @Test
    @DisplayName("a folder that is not a repository is refused with git's reason, and nothing is written")
    void aPlainFolderIsRefused(@TempDir Path folder) throws Exception {
        Files.writeString(folder.resolve("a.txt"), "a");

        assertThatThrownBy(() -> committer.pendingChanges(folder))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("Could not read the working tree");
        assertThatThrownBy(() -> committer.commitAll(folder, "feat: nowhere"))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("Could not stage");
        assertThat(Files.exists(folder.resolve(".git"))).isFalse();
    }

    private void initWithIdentity(Path repo) throws IOException, InterruptedException {
        run(repo, "init", "-q");
        run(repo, "config", "user.name", "Test");
        run(repo, "config", "user.email", "test@example.com");
    }

    private String run(Path directory, String... args) throws IOException, InterruptedException {
        List<String> command = new ArrayList<>(List.of("git", "-C", directory.toString()));
        command.addAll(List.of(args));
        Process process = new ProcessBuilder(command).start();
        String output = new String(process.getInputStream().readAllBytes()).strip();
        process.waitFor(5, TimeUnit.SECONDS);
        return output;
    }
}
