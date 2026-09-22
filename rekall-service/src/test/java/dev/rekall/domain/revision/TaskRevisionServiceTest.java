package dev.rekall.domain.revision;

import dev.rekall.domain.RevisionKind;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskRevision;
import dev.rekall.domain.WrapupAuthor;
import dev.rekall.domain.repository.TaskRevisionRepository;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.mockito.ArgumentCaptor;

import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.ZoneOffset;
import java.util.List;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.IntStream;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

class TaskRevisionServiceTest {

    private static final Instant NOW = Instant.parse("2026-09-22T10:00:00Z");

    private final TaskRevisionRepository repository = mock(TaskRevisionRepository.class);
    private final TaskRevisionService service =
            new TaskRevisionService(repository, Clock.fixed(NOW, ZoneOffset.UTC));

    private final UUID taskId = UUID.randomUUID();
    private Task task;

    @BeforeEach
    void setUp() {
        task = mock(Task.class);
        when(task.getId()).thenReturn(taskId);
        when(repository.findFirstByTaskIdAndKindOrderByCreatedAtDesc(taskId, RevisionKind.WRAPUP))
                .thenReturn(Optional.empty());
        when(repository.findByTaskIdAndKindOrderByCreatedAtDesc(taskId, RevisionKind.WRAPUP)).thenReturn(List.of());
    }

    @Test
    @DisplayName("the text a session's write replaces is kept, with who wrote it and when")
    void aClaudeWriteKeepsTheOutgoingText() {
        Instant writtenAt = NOW.minus(Duration.ofDays(1));

        boolean kept = service.keep(task, RevisionKind.WRAPUP, "old wrapup", WrapupAuthor.HAND, writtenAt,
                RevisionTrigger.CLAUDE_WRITE);

        assertThat(kept).isTrue();
        ArgumentCaptor<TaskRevision> saved = ArgumentCaptor.forClass(TaskRevision.class);
        verify(repository).saveAndFlush(saved.capture());
        assertThat(saved.getValue().getBodyMarkdown()).isEqualTo("old wrapup");
        assertThat(saved.getValue().getWrittenBy()).isEqualTo(WrapupAuthor.HAND);
        assertThat(saved.getValue().getWrittenAt()).isEqualTo(writtenAt);
    }

    @Test
    @DisplayName("blank text, or text the newest revision already holds, is not kept")
    void blankAndDuplicateTextIsNotKept() {
        assertThat(service.keep(task, RevisionKind.WRAPUP, "  ", null, null, RevisionTrigger.DELETION)).isFalse();

        newestIs("same text", NOW.minus(Duration.ofDays(3)));
        assertThat(service.keep(task, RevisionKind.WRAPUP, "same text", null, null, RevisionTrigger.CLAUDE_WRITE))
                .isFalse();

        verify(repository, never()).saveAndFlush(any());
    }

    @Test
    @DisplayName("hand edits over hand-written text keep one revision per ten-minute window, not one per keystroke")
    void handEditsAreCoalesced() {
        newestIs("before typing", NOW.minus(Duration.ofMinutes(3)));

        assertThat(service.keep(task, RevisionKind.WRAPUP, "half typed", WrapupAuthor.HAND, null,
                RevisionTrigger.HAND_EDIT)).isFalse();

        newestIs("before typing", NOW.minus(Duration.ofMinutes(11)));
        assertThat(service.keep(task, RevisionKind.WRAPUP, "an hour later", WrapupAuthor.HAND, null,
                RevisionTrigger.HAND_EDIT)).isTrue();
    }

    @Test
    @DisplayName("a hand edit over a session's text is always kept, however recent the last revision")
    void aHandEditOverClaudeTextIsKept() {
        newestIs("older", NOW.minus(Duration.ofMinutes(1)));

        assertThat(service.keep(task, RevisionKind.WRAPUP, "claude's version", WrapupAuthor.CLAUDE, null,
                RevisionTrigger.HAND_EDIT)).isTrue();
    }

    @Test
    @DisplayName("a deletion and a restore are always kept, even inside the editing window")
    void deletionsAndRestoresAreAlwaysKept() {
        newestIs("older", NOW.minus(Duration.ofMinutes(1)));

        assertThat(service.keep(task, RevisionKind.WRAPUP, "deleted", WrapupAuthor.HAND, null,
                RevisionTrigger.DELETION)).isTrue();
        assertThat(service.keep(task, RevisionKind.WRAPUP, "restored over", WrapupAuthor.HAND, null,
                RevisionTrigger.RESTORE)).isTrue();
    }

    @Test
    @DisplayName("past thirty revisions of one text, the oldest are dropped")
    void theOldestArePruned() {
        List<TaskRevision> all = IntStream.range(0, TaskRevisionService.KEPT_PER_KIND + 2)
                .mapToObj(at -> mock(TaskRevision.class))
                .toList();
        when(repository.findByTaskIdAndKindOrderByCreatedAtDesc(taskId, RevisionKind.WRAPUP)).thenReturn(all);

        service.keep(task, RevisionKind.WRAPUP, "one more", null, null, RevisionTrigger.CLAUDE_WRITE);

        verify(repository).deleteAll(all.subList(TaskRevisionService.KEPT_PER_KIND, all.size()));
    }

    private void newestIs(String body, Instant createdAt) {
        TaskRevision newest = mock(TaskRevision.class);
        when(newest.getBodyMarkdown()).thenReturn(body);
        when(newest.getCreatedAt()).thenReturn(createdAt);
        when(repository.findFirstByTaskIdAndKindOrderByCreatedAtDesc(taskId, RevisionKind.WRAPUP))
                .thenReturn(Optional.of(newest));
    }
}
