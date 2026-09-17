package dev.rekall.domain.commit;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.stream.IntStream;

import static org.assertj.core.api.Assertions.assertThat;

class CommitMessageGeneratorTest {

    private static final CommitMessageGenerator.Subject STEP =
            new CommitMessageGenerator.Subject("Wire the export endpoint", "project:vega task:report-builder", 3);

    @Test
    @DisplayName("a mixed change is a feat, with the step title as the subject")
    void aMixedChangeIsAFeat() {
        List<PendingChange> changes = List.of(
                new PendingChange(PendingChange.Kind.ADDED, "src/export/ExportController.java"),
                new PendingChange(PendingChange.Kind.MODIFIED, "README.md"));

        String message = CommitMessageGenerator.generate(STEP, changes);

        assertThat(message).startsWith("feat: Wire the export endpoint\n\n");
        assertThat(message).contains("project:vega task:report-builder, step 3\n");
        assertThat(message).contains("2 files: 1 added, 1 modified\n");
        assertThat(message).contains("  + src/export/ExportController.java\n").contains("  ~ README.md\n");
    }

    @Test
    @DisplayName("only documentation files make a docs commit")
    void onlyDocumentationMakesADocsCommit() {
        List<PendingChange> changes = List.of(
                new PendingChange(PendingChange.Kind.MODIFIED, "README.md"),
                new PendingChange(PendingChange.Kind.ADDED, "docs/export.txt"));

        assertThat(CommitMessageGenerator.typeOf(changes)).isEqualTo("docs");
    }

    @Test
    @DisplayName("only test files make a test commit")
    void onlyTestFilesMakeATestCommit() {
        List<PendingChange> changes = List.of(
                new PendingChange(PendingChange.Kind.ADDED, "src/test/java/dev/vega/ExportTest.java"),
                new PendingChange(PendingChange.Kind.MODIFIED, "ui/tests/unit/Export.spec.ts"));

        assertThat(CommitMessageGenerator.typeOf(changes)).isEqualTo("test");
    }

    @Test
    @DisplayName("a test next to a source file is a feat, not a test")
    void aTestNextToSourceIsAFeat() {
        List<PendingChange> changes = List.of(
                new PendingChange(PendingChange.Kind.ADDED, "src/test/java/dev/vega/ExportTest.java"),
                new PendingChange(PendingChange.Kind.ADDED, "src/main/java/dev/vega/Export.java"));

        assertThat(CommitMessageGenerator.typeOf(changes)).isEqualTo("feat");
    }

    @Test
    @DisplayName("a task claim carries no step number and names the task")
    void aTaskClaimNamesTheTask() {
        CommitMessageGenerator.Subject task =
                new CommitMessageGenerator.Subject("Report builder", "project:vega task:report-builder", null);

        String message = CommitMessageGenerator.generate(
                task, List.of(new PendingChange(PendingChange.Kind.DELETED, "old.js")));

        assertThat(message).startsWith("feat: Report builder\n\nproject:vega task:report-builder\n1 file: 1 deleted\n  - old.js\n");
    }

    @Test
    @DisplayName("the subject line is cut at 72 characters, with the title ellipsised")
    void theSubjectLineIsCut() {
        CommitMessageGenerator.Subject longTitle = new CommitMessageGenerator.Subject(
                "A".repeat(100), "project:vega task:report-builder", 1);

        String subject = CommitMessageGenerator.subjectLine(longTitle, List.of());

        assertThat(subject).hasSize(CommitMessageGenerator.SUBJECT_MAX);
        assertThat(subject).startsWith("feat: AAAA").endsWith("…");
    }

    @Test
    @DisplayName("past twenty paths the rest are counted, not listed")
    void pastTwentyPathsTheRestAreCounted() {
        List<PendingChange> changes = IntStream.range(0, 25)
                .mapToObj(at -> new PendingChange(PendingChange.Kind.ADDED, "src/file" + at + ".ts"))
                .toList();

        String message = CommitMessageGenerator.generate(STEP, changes);

        assertThat(message).contains("25 files: 25 added").contains("  + src/file19.ts\n  … and 5 more\n");
        assertThat(message).doesNotContain("src/file20.ts");
    }

    @Test
    @DisplayName("a whitespace-heavy title is collapsed to single spaces")
    void aWhitespaceHeavyTitleIsCollapsed() {
        CommitMessageGenerator.Subject messy =
                new CommitMessageGenerator.Subject("  Wire   the\nexport  ", "project:vega task:x", 1);

        assertThat(CommitMessageGenerator.subjectLine(messy, List.of())).isEqualTo("feat: Wire the export");
    }
}
