package dev.rekall.domain.note;

import dev.rekall.domain.Document;
import dev.rekall.domain.Task;
import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.DocumentRepository;
import dev.rekall.domain.repository.TaskRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;

/**
 * The one way a session writes a note: a new one, on one task. It cannot open a note that is
 * already there, so it cannot edit, move, detach or delete one; the console's
 * {@code DocumentService} stays the only path for those, and off the MCP classpath.
 */
@Service
@RequiredArgsConstructor
public class NoteService {

    public static final String KIND = "notes";

    static final int TITLE_MAX_CHARACTERS = 255;

    static final int BODY_MAX_CHARACTERS = 100_000;

    private final TaskRepository tasks;
    private final DocumentRepository documents;
    private final ApplicationEventPublisher events;

    public record Written(String title, String noteAnchor, String taskAnchor, int notesOnTask) {
    }

    @Transactional
    public Written write(String projectLabel, String taskLabel, String title, String body) {
        Task task = resolveTask(projectLabel, taskLabel);
        String wanted = validatedTitle(title);
        String text = validatedBody(body);
        task.getDocuments().stream()
                .filter(existing -> existing.getTitle().strip().equalsIgnoreCase(wanted))
                .findFirst()
                .ifPresent(existing -> {
                    throw new IllegalArgumentException(
                            "This task already has a note titled '%s' (%s). Nothing was written: a session "
                                    .formatted(existing.getTitle(), existing.anchor())
                                    + "only adds notes, so pick another title, or leave changing that one to a "
                                    + "person in the console.");
                });

        Document document = documents.save(new Document(wanted, KIND, text));
        task.attach(document);
        documents.flush();
        events.publishEvent(new NoteStreamEvent(task.getId(), document.getId()));
        return new Written(
                document.getTitle(),
                document.anchor(),
                "project:%s task:%s".formatted(task.getProject().getLabel(), task.getLabel()),
                task.getDocuments().size());
    }

    private String validatedTitle(String title) {
        if (title == null || title.isBlank()) {
            throw new IllegalArgumentException("A note needs a title.");
        }
        String text = title.strip();
        if (text.length() > TITLE_MAX_CHARACTERS) {
            throw new IllegalArgumentException("A note's title is capped at %d characters and this one is %d."
                    .formatted(TITLE_MAX_CHARACTERS, text.length()));
        }
        return text;
    }

    private String validatedBody(String body) {
        if (body == null || body.isBlank()) {
            throw new IllegalArgumentException(
                    "A note needs a body. An empty one is something a person makes in the console.");
        }
        String text = body.strip();
        if (text.length() > BODY_MAX_CHARACTERS) {
            throw new IllegalArgumentException("A note is capped at %d characters and this one is %d. Split it."
                    .formatted(BODY_MAX_CHARACTERS, text.length()));
        }
        return text;
    }

    private Task resolveTask(String projectLabel, String taskLabel) {
        if (projectLabel != null) {
            return tasks.findByProjectLabelIgnoreCaseAndLabelIgnoreCase(projectLabel, taskLabel)
                    .orElseThrow(() -> new UnknownAnchorException(
                            "No task '%s' on project '%s'".formatted(taskLabel, projectLabel)));
        }
        List<Task> found = tasks.findByLabelIgnoreCase(taskLabel);
        if (found.isEmpty()) {
            throw new UnknownAnchorException("No task matches '%s'".formatted(taskLabel));
        }
        if (found.size() > 1) {
            throw new AmbiguousAnchorException(
                    taskLabel, found.stream().map(task -> "project:" + task.getProject().getLabel()).toList());
        }
        return found.getFirst();
    }
}
