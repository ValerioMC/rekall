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

class GitRepositoryInspectorTest {

    private final GitRepositoryInspector inspector = new GitRepositoryInspector();

    @Test
    @DisplayName("no folder at all is unset, not missing")
    void noFolderIsUnset() {
        assertThat(inspector.inspect(null)).isEqualTo(RepositoryStatus.unset());
        assertThat(inspector.inspect("   ")).isEqualTo(RepositoryStatus.unset());
    }

    @Test
    @DisplayName("a folder that is not there is missing")
    void aFolderThatIsNotThereIsMissing(@TempDir Path dir) {
        String folder = dir.resolve("nope").toString();

        RepositoryStatus status = inspector.inspect(folder);

        assertThat(status).isEqualTo(RepositoryStatus.missing(folder));
        assertThat(inspector.isRepository(folder)).isFalse();
    }

    @Test
    @DisplayName("a plain folder exists but is not a repository")
    void aPlainFolderIsNotARepository(@TempDir Path dir) {
        RepositoryStatus status = inspector.inspect(dir.toString());

        assertThat(status.exists()).isTrue();
        assertThat(status.repository()).isFalse();
        assertThat(status.canCommit()).isFalse();
    }

    @Test
    @DisplayName("a repository reports its branch and the identity git resolves inside it")
    void aRepositoryReportsBranchAndIdentity(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q", "-b", "main");
        run(repo, "config", "user.name", "Test Person");
        run(repo, "config", "user.email", "test@example.com");
        Files.writeString(repo.resolve("a.txt"), "a");
        run(repo, "add", "-A");
        run(repo, "commit", "-q", "-m", "seed");

        RepositoryStatus status = inspector.inspect(" " + repo + " ");

        assertThat(status.folder()).isEqualTo(repo.toString());
        assertThat(status.repository()).isTrue();
        assertThat(status.branch()).isEqualTo("main");
        assertThat(status.userName()).isEqualTo("Test Person");
        assertThat(status.userEmail()).isEqualTo("test@example.com");
        assertThat(status.canCommit()).isTrue();
    }

    @Test
    @DisplayName("a repository with no commit yet still names the branch it will commit on")
    void anEmptyRepositoryStillNamesItsBranch(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q", "-b", "trunk");

        assertThat(inspector.inspect(repo.toString()).branch()).isEqualTo("trunk");
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
