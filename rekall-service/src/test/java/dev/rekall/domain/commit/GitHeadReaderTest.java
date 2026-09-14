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

/**
 * Exercises the real {@code git} binary against a throwaway repository: this class is a thin
 * process wrapper, so mocking it out would test nothing.
 */
class GitHeadReaderTest {

    private final GitHeadReader reader = new GitHeadReader();

    @Test
    @DisplayName("reads the hash and subject of the tip commit")
    void readsTheHashAndSubjectOfTheTipCommit(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("README.md"), "hello");
        run(repo, "add", "README.md");
        commit(repo, "Add the readme");

        GitHeadReader.Commit head = reader.head(repo);

        assertThat(head.hash()).isEqualTo(run(repo, "rev-parse", "HEAD"));
        assertThat(head.subject()).isEqualTo("Add the readme");
    }

    @Test
    @DisplayName("reads the diff the tip commit introduced")
    void readsTheDiffTheTipCommitIntroduced(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("README.md"), "hello");
        run(repo, "add", "README.md");
        commit(repo, "Add the readme");

        GitHeadReader.Commit head = reader.head(repo);

        assertThat(head.diff()).contains("+hello").contains("README.md");
    }

    @Test
    @DisplayName("the diff of a second commit is only what it changed, not the whole file again")
    void readsOnlyWhatTheSecondCommitChanged(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("a.txt"), "one\n");
        run(repo, "add", "a.txt");
        commit(repo, "Add a.txt");
        Files.writeString(repo.resolve("a.txt"), "one\ntwo\n");
        run(repo, "add", "a.txt");
        commit(repo, "Append a second line");

        GitHeadReader.Commit head = reader.head(repo);

        assertThat(head.diff()).contains("+two").doesNotContain("+one");
    }

    @Test
    @DisplayName("only the subject line is kept, even when the commit has a body")
    void readsOnlyTheSubjectLineEvenWithABody(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("a.txt"), "a");
        run(repo, "add", "a.txt");
        commit(repo, "Short subject\n\nA much longer body explaining why.");

        GitHeadReader.Commit head = reader.head(repo);

        assertThat(head.subject()).isEqualTo("Short subject");
    }

    @Test
    @DisplayName("a repository with no commits yet is refused, not read as an empty hash")
    void refusesARepositoryWithNoCommitsYet(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");

        assertThatThrownBy(() -> reader.head(repo)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    @DisplayName("a folder that is not a git repository is refused")
    void refusesAFolderThatIsNotAGitRepository(@TempDir Path notARepo) {
        assertThatThrownBy(() -> reader.head(notARepo)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    @DisplayName("a folder that does not exist is refused with a clear reason")
    void refusesAFolderThatDoesNotExist(@TempDir Path parent) {
        Path missing = parent.resolve("does-not-exist");

        assertThatThrownBy(() -> reader.head(missing))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("not there");
    }

    private void commit(Path repo, String message) throws IOException, InterruptedException {
        run(repo, "-c", "user.name=Test", "-c", "user.email=test@example.com",
                "commit", "-q", "-m", message);
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
