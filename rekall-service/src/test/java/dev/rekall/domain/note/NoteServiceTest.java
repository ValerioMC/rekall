package dev.rekall.domain.note;

import dev.rekall.domain.Document;
import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.repository.DocumentRepository;
import dev.rekall.domain.repository.TaskRepository;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.context.ApplicationEventPublisher;

import java.util.HashSet;
import java.util.List;
import java.util.Optional;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

class NoteServiceTest {

    private final TaskRepository tasks = mock(TaskRepository.class);
    private final DocumentRepository documents = mock(DocumentRepository.class);
    private final ApplicationEventPublisher events = mock(ApplicationEventPublisher.class);
    private final NoteService service = new NoteService(tasks, documents, events);

    @Test
    @DisplayName("a title the task's notes already carry is refused, whatever its case, and nothing is saved")
    void refusesATitleTheTaskAlreadyCarries() {
        Task task = taskOnVega("report-builder");
        Document existing = mock(Document.class);
        when(existing.getTitle()).thenReturn("Export-Formats.md");
        when(existing.anchor()).thenReturn("note:3f2a9c1e");
        when(existing.getTasks()).thenReturn(new HashSet<>());
        task.attach(existing);
        when(tasks.findByProjectLabelIgnoreCaseAndLabelIgnoreCase("vega", "report-builder"))
                .thenReturn(Optional.of(task));

        assertThatThrownBy(() -> service.write("vega", "report-builder", " export-formats.md ", "new"))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("already has a note titled 'Export-Formats.md'");
        verify(documents, never()).save(any());
        verify(events, never()).publishEvent(any());
    }

    @Test
    @DisplayName("a blank body is refused: an empty note is the console's to make")
    void refusesABlankBody() {
        when(tasks.findByProjectLabelIgnoreCaseAndLabelIgnoreCase("vega", "report-builder"))
                .thenReturn(Optional.of(taskOnVega("report-builder")));

        assertThatThrownBy(() -> service.write("vega", "report-builder", "x.md", "   "))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("needs a body");
    }

    @Test
    @DisplayName("a bare task label two projects share is reported ambiguous rather than picked")
    void reportsABareLabelTwoProjectsShare() {
        when(tasks.findByLabelIgnoreCase("report-builder"))
                .thenReturn(List.of(taskOnVega("report-builder"), taskOnVega("report-builder")));

        assertThatThrownBy(() -> service.write(null, "report-builder", "x.md", "body"))
                .isInstanceOf(AmbiguousAnchorException.class);
    }

    private Task taskOnVega(String label) {
        Task task = new Task(label, label);
        task.setProject(new Project("vega", "Vega"));
        return task;
    }
}
