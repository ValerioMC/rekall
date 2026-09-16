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
class GitLogReaderTest {

    private final GitLogReader reader = new GitLogReader();

    @Test
    @DisplayName("reads the hash and subject of the tip commit")
    void readsTheHashAndSubjectOfTheTipCommit(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("README.md"), "hello");
        run(repo, "add", "README.md");
        commit(repo, "Add the readme");

        GitLogReader.Commit head = reader.head(repo);

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

        GitLogReader.Commit head = reader.head(repo);

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

        GitLogReader.Commit head = reader.head(repo);

        assertThat(head.diff()).contains("+two").doesNotContain("+one");
    }

    @Test
    @DisplayName("only the subject line is kept, even when the commit has a body")
    void readsOnlyTheSubjectLineEvenWithABody(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("a.txt"), "a");
        run(repo, "add", "a.txt");
        commit(repo, "Short subject\n\nA much longer body explaining why.");

        GitLogReader.Commit head = reader.head(repo);

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

    @Test
    @DisplayName("a past commit is read by its full hash, with its own subject and diff")
    void readsAPastCommitByItsFullHash(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("a.txt"), "one\n");
        run(repo, "add", "a.txt");
        commit(repo, "Add a.txt");
        String first = run(repo, "rev-parse", "HEAD");
        Files.writeString(repo.resolve("a.txt"), "one\ntwo\n");
        run(repo, "add", "a.txt");
        commit(repo, "Append a second line");

        GitLogReader.Commit past = reader.commit(repo, first);

        assertThat(past.hash()).isEqualTo(first);
        assertThat(past.subject()).isEqualTo("Add a.txt");
        assertThat(past.diff()).contains("+one").doesNotContain("+two");
    }

    @Test
    @DisplayName("an abbreviated hash resolves to the full one, the way git log prints it")
    void readsACommitByAnAbbreviatedHash(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("a.txt"), "a");
        run(repo, "add", "a.txt");
        commit(repo, "Add a.txt");
        String full = run(repo, "rev-parse", "HEAD");

        GitLogReader.Commit found = reader.commit(repo, full.substring(0, 7));

        assertThat(found.hash()).isEqualTo(full);
    }

    @Test
    @DisplayName("a hash that names no commit is refused with the hash in the reason")
    void refusesAHashThatNamesNoCommit(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("a.txt"), "a");
        run(repo, "add", "a.txt");
        commit(repo, "Add a.txt");

        assertThatThrownBy(() -> reader.commit(repo, "deadbeef"))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("deadbeef");
    }

    @Test
    @DisplayName("anything that is not hex is refused before git is even asked")
    void refusesAReferenceThatIsNotAHash(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("a.txt"), "a");
        run(repo, "add", "a.txt");
        commit(repo, "Add a.txt");

        assertThatThrownBy(() -> reader.commit(repo, "--output=/tmp/x"))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("not a commit hash");
        assertThatThrownBy(() -> reader.commit(repo, "main"))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("not a commit hash");
    }

    @Test
    @DisplayName("the recent log comes back newest first, with each commit's date")
    void readsTheRecentLogNewestFirst(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        Files.writeString(repo.resolve("a.txt"), "one\n");
        run(repo, "add", "a.txt");
        commit(repo, "First");
        Files.writeString(repo.resolve("a.txt"), "two\n");
        run(repo, "add", "a.txt");
        commit(repo, "Second");
        Files.writeString(repo.resolve("a.txt"), "three\n");
        run(repo, "add", "a.txt");
        commit(repo, "Third");

        List<GitLogReader.LogEntry> log = reader.recent(repo, 10);

        assertThat(log).extracting(GitLogReader.LogEntry::subject).containsExactly("Third", "Second", "First");
        assertThat(log.getFirst().hash()).isEqualTo(run(repo, "rev-parse", "HEAD"));
        assertThat(log).allSatisfy(entry -> assertThat(entry.committedAt()).isNotNull());
    }

    @Test
    @DisplayName("the recent log stops at the limit it was asked for")
    void theRecentLogStopsAtTheLimit(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");
        for (int i = 1; i <= 4; i++) {
            Files.writeString(repo.resolve("a.txt"), "line " + i + "\n");
            run(repo, "add", "a.txt");
            commit(repo, "Commit " + i);
        }

        assertThat(reader.recent(repo, 2))
                .extracting(GitLogReader.LogEntry::subject)
                .containsExactly("Commit 4", "Commit 3");
    }

    @Test
    @DisplayName("the recent log of a repository with no commits yet is refused, not empty")
    void theRecentLogOfAnEmptyRepositoryIsRefused(@TempDir Path repo) throws Exception {
        run(repo, "init", "-q");

        assertThatThrownBy(() -> reader.recent(repo, 10)).isInstanceOf(IllegalArgumentException.class);
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
