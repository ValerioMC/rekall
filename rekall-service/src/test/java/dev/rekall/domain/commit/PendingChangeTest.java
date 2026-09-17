package dev.rekall.domain.commit;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

class PendingChangeTest {

    @Test
    @DisplayName("an untracked file reads as added, since git add -A is about to stage it")
    void anUntrackedFileReadsAsAdded() {
        assertThat(PendingChange.parse("?? src/new.ts"))
                .isEqualTo(new PendingChange(PendingChange.Kind.ADDED, "src/new.ts"));
    }

    @Test
    @DisplayName("a staged addition and a tracked modification keep their kind")
    void stagedAndModifiedKeepTheirKind() {
        assertThat(PendingChange.parse("A  src/staged.ts").kind()).isEqualTo(PendingChange.Kind.ADDED);
        assertThat(PendingChange.parse(" M src/edited.ts").kind()).isEqualTo(PendingChange.Kind.MODIFIED);
        assertThat(PendingChange.parse("MM src/both.ts").kind()).isEqualTo(PendingChange.Kind.MODIFIED);
    }

    @Test
    @DisplayName("a deletion, staged or not, reads as deleted")
    void aDeletionReadsAsDeleted() {
        assertThat(PendingChange.parse(" D gone.ts").kind()).isEqualTo(PendingChange.Kind.DELETED);
        assertThat(PendingChange.parse("D  gone.ts").kind()).isEqualTo(PendingChange.Kind.DELETED);
    }

    @Test
    @DisplayName("a rename keeps the new path")
    void aRenameKeepsTheNewPath() {
        assertThat(PendingChange.parse("R  old.ts -> new.ts"))
                .isEqualTo(new PendingChange(PendingChange.Kind.RENAMED, "new.ts"));
    }

    @Test
    @DisplayName("a line too short to be a status line is refused")
    void aShortLineIsRefused() {
        assertThatThrownBy(() -> PendingChange.parse("M "))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("Unexpected git status line");
    }
}
