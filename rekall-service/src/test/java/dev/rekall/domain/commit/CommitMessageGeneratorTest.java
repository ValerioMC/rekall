package dev.rekall.domain.commit;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

class CommitMessageGeneratorTest {

    private static final String ANCHOR = "project:vega task:report-builder";

    private static final String DETAIL = """
            ## What this step does

            Expose **the weekly report** as a download, so the finance team stops asking for it by mail.
            The endpoint streams the file instead of building it in memory.

            ```java
            ignored();
            ```

            It is read-only and needs no migration.
            """;

    private static final CommitMessageGenerator.Subject STEP =
            new CommitMessageGenerator.Subject("Wire the export endpoint", ANCHOR, 3, DETAIL);

    private static final List<PendingChange> MIXED = List.of(
            new PendingChange(PendingChange.Kind.ADDED, "src/export/ExportController.java"),
            new PendingChange(PendingChange.Kind.MODIFIED, "README.md"));

    @Test
    @DisplayName("without a session message, the body is the step's detail as plain prose, and no file is listed")
    void theDerivedBodyIsTheDetailAsProse() {
        String message = CommitMessageGenerator.generate(STEP, MIXED, null);

        assertThat(message).startsWith("feat: Wire the export endpoint\n\n");
        assertThat(message)
                .contains("Expose the weekly report as a download")
                .contains("It is read-only and needs no migration.")
                .doesNotContain("##")
                .doesNotContain("**")
                .doesNotContain("ignored();");
        assertThat(message).doesNotContain("ExportController.java").doesNotContain("README.md").doesNotContain("files");
        assertThat(message).endsWith("\n\nRefs: project:vega task:report-builder, step 3\n");
    }

    @Test
    @DisplayName("a session message wins: its first line is the subject and the rest is the body")
    void aSessionMessageWins() {
        String written = """
                fix: stream the weekly report instead of buffering it

                The export built the whole file in memory and ran out of heap on large months.
                It now writes rows as they are read.
                """;

        String message = CommitMessageGenerator.generate(STEP, MIXED, written);

        assertThat(message).startsWith("fix: stream the weekly report instead of buffering it\n\n");
        assertThat(message)
                .as("the session's lines are reflowed at 72 columns")
                .contains("ran out of heap on large\nmonths. It now writes rows as they are read.")
                .doesNotContain("Expose the weekly");
        assertThat(message).endsWith("Refs: project:vega task:report-builder, step 3\n");
    }

    @Test
    @DisplayName("a session subject without a type gets one, and a session subject alone keeps the derived body")
    void aBareSessionSubjectIsTyped() {
        String message = CommitMessageGenerator.generate(STEP, MIXED, "Stream the weekly report");

        assertThat(message).startsWith("feat: Stream the weekly report\n\n").contains("Expose the weekly report");
    }

    @Test
    @DisplayName("body lines are wrapped at 72 columns, list items keep a hanging indent")
    void bodyLinesAreWrapped() {
        String written = "feat: x\n\n" + "word ".repeat(40) + "\n\n- " + "item ".repeat(30);

        String message = CommitMessageGenerator.generate(STEP, MIXED, written);

        assertThat(Arrays.stream(message.split("\n"))).allMatch(line -> line.length() <= 72);
        assertThat(message).contains("\n- item").contains("\n  item");
    }

    @Test
    @DisplayName("a long detail is cut at a sentence, not mid-word")
    void aLongDetailIsCutAtASentence() {
        String detail = "This sentence is long enough to matter. ".repeat(40);

        String body = CommitMessageGenerator.derivedBody(detail);

        assertThat(body.length()).isLessThanOrEqualTo(CommitMessageGenerator.DERIVED_BODY_MAX);
        assertThat(body).endsWith("matter.");
    }

    @Test
    @DisplayName("a task claim with no wrapup has a subject and the refs line, nothing else")
    void aTaskClaimWithNoWrapup() {
        CommitMessageGenerator.Subject task = new CommitMessageGenerator.Subject("Report builder", ANCHOR, null, null);

        String message = CommitMessageGenerator.generate(
                task, List.of(new PendingChange(PendingChange.Kind.DELETED, "old.js")), null);

        assertThat(message).isEqualTo("feat: Report builder\n\nRefs: project:vega task:report-builder\n");
    }

    @Test
    @DisplayName("only documentation files make a docs commit, only tests a test commit")
    void theFilesDecideDocsAndTest() {
        assertThat(CommitMessageGenerator.typeOf("Update guide", List.of(
                new PendingChange(PendingChange.Kind.MODIFIED, "README.md"),
                new PendingChange(PendingChange.Kind.ADDED, "docs/export.txt")))).isEqualTo("docs");
        assertThat(CommitMessageGenerator.typeOf("Cover export", List.of(
                new PendingChange(PendingChange.Kind.ADDED, "src/test/java/dev/vega/ExportTest.java"),
                new PendingChange(PendingChange.Kind.MODIFIED, "ui/tests/unit/Export.spec.ts")))).isEqualTo("test");
        assertThat(CommitMessageGenerator.typeOf("Cover export", List.of(
                new PendingChange(PendingChange.Kind.ADDED, "src/test/java/dev/vega/ExportTest.java"),
                new PendingChange(PendingChange.Kind.ADDED, "src/main/java/dev/vega/Export.java")))).isEqualTo("feat");
    }

    @Test
    @DisplayName("the title decides fix and refactor, in English or Italian")
    void theTitleDecidesFixAndRefactor() {
        assertThat(CommitMessageGenerator.typeOf("Fix the await review", MIXED)).isEqualTo("fix");
        assertThat(CommitMessageGenerator.typeOf("Errore quando faccio accetto", MIXED)).isEqualTo("fix");
        assertThat(CommitMessageGenerator.typeOf("Refactor the store", MIXED)).isEqualTo("refactor");
        assertThat(CommitMessageGenerator.typeOf("Fixture loader", MIXED)).isEqualTo("feat");
    }

    @Test
    @DisplayName("the subject line is cut at 72 characters, with the title ellipsised")
    void theSubjectLineIsCut() {
        CommitMessageGenerator.Subject longTitle = new CommitMessageGenerator.Subject("A".repeat(100), ANCHOR, 1, null);

        String subject = CommitMessageGenerator.subjectLine(longTitle, List.of());

        assertThat(subject).hasSize(CommitMessageGenerator.SUBJECT_MAX);
        assertThat(subject).startsWith("feat: AAAA").endsWith("…");
    }

    @Test
    @DisplayName("a whitespace-heavy title is collapsed to single spaces")
    void aWhitespaceHeavyTitleIsCollapsed() {
        CommitMessageGenerator.Subject messy = new CommitMessageGenerator.Subject("  Wire   the\nexport  ", ANCHOR, 1, null);

        assertThat(CommitMessageGenerator.subjectLine(messy, List.of())).isEqualTo("feat: Wire the export");
    }
}
