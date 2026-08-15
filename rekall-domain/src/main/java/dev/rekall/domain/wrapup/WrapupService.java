package dev.rekall.domain.wrapup;

import dev.rekall.domain.Task;
import dev.rekall.domain.Wrapup;
import dev.rekall.domain.WrapupAuthor;
import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.WrapupRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * Reads and replaces the one wrapup a task may have.
 *
 * <p>This is the only write service in the domain, and it exists because the wrapup is the one
 * thing Claude has to be able to put back. Everything it can do is one row of one table: given
 * a task, replace its wrapup body, or read it. It cannot create a task, rename one, touch a
 * note or reach any other column, which is what keeps the widening from D2 to a single shape
 * rather than a door. {@code docs/DESIGN.md} §8 records the trade.
 *
 * <p>Replace, never append. A wrapup is the state of the implementation as it stands, so the
 * caller sends the whole text and it overwrites what was there. Nothing here merges, diffs or
 * keeps a previous version: a wrapup that accumulated would become the log it is defined
 * against.
 */
@Service
@RequiredArgsConstructor
public class WrapupService {

    private final TaskRepository tasks;
    private final WrapupRepository wrapups;

    /**
     * The result of a write, and what it displaced.
     *
     * @param created true the first time a task gets a wrapup, false every time after
     * @param replaced who had written the version this one overwrote, or null on the first
     *     write. Carried because overwriting a wrapup you corrected by hand is the one outcome
     *     worth saying out loud
     */
    public record Written(WrapupView wrapup, boolean created, WrapupAuthor replaced) {
    }

    // ------------------------------------------------------------------ reading

    @Transactional(readOnly = true)
    public Optional<WrapupView> find(UUID taskId) {
        return wrapups.findByTaskId(taskId).map(WrapupView::of);
    }

    @Transactional(readOnly = true)
    public List<WrapupView> findAll() {
        return wrapups.findAllByOrderByUpdatedAtDesc().stream().map(WrapupView::of).toList();
    }

    /**
     * The anchored read, for a caller holding {@code project:vega task:report-builder} rather
     * than an id.
     */
    @Transactional(readOnly = true)
    public Optional<WrapupView> find(String projectLabel, String taskLabel) {
        return find(resolve(projectLabel, taskLabel).getId());
    }

    // ------------------------------------------------------------------ writing

    /** The anchored write, which is the form the MCP tool uses. */
    @Transactional
    public Written write(String projectLabel, String taskLabel, String body, WrapupAuthor author) {
        return write(resolve(projectLabel, taskLabel), body, author);
    }

    @Transactional
    public Written write(UUID taskId, String body, WrapupAuthor author) {
        return write(
                tasks.findById(taskId)
                        .orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId)),
                body,
                author);
    }

    private Written write(Task task, String body, WrapupAuthor author) {
        String text = validated(body);
        Optional<Wrapup> existing = wrapups.findByTaskId(task.getId());
        if (existing.isEmpty()) {
            Wrapup wrapup = new Wrapup(task, text, author);
            task.setWrapup(wrapup);
            return new Written(WrapupView.of(wrapups.saveAndFlush(wrapup)), true, null);
        }
        Wrapup wrapup = existing.get();
        WrapupAuthor previous = wrapup.getWrittenBy();
        wrapup.setBodyMarkdown(text);
        wrapup.setWrittenBy(author);
        return new Written(WrapupView.of(wrapups.saveAndFlush(wrapup)), false, previous);
    }

    /** Clearing a task's wrapup. The task keeps everything else. */
    @Transactional
    public void delete(UUID taskId) {
        wrapups.findByTaskId(taskId).ifPresent(wrapup -> {
            wrapup.getTask().setWrapup(null);
            wrapups.delete(wrapup);
        });
    }

    // ------------------------------------------------------------------ checks

    /**
     * The two things a wrapup body cannot be: absent, or long enough to have become a log.
     *
     * <p>Both are refused here rather than at the column, so the message says what to do about
     * it. What a wrapup must contain — the state and not the story of how it got there — is not
     * checkable and is not attempted; it is stated where it can be read, in the tool description
     * and in the slash command.
     */
    private String validated(String body) {
        if (body == null || body.isBlank()) {
            throw new IllegalArgumentException(
                    "A wrapup needs a body. To remove one, delete it rather than blanking it.");
        }
        String text = body.strip();
        if (text.length() > Wrapup.MAX_CHARACTERS) {
            throw new IllegalArgumentException(
                    "A wrapup is capped at %d characters and this one is %d. It describes the state of the "
                            .formatted(Wrapup.MAX_CHARACTERS, text.length())
                            + "implementation, not how it got there; if it does not fit, it is recording the "
                            + "process. Move the detail into a note.");
        }
        return text;
    }

    /**
     * Resolves the task an anchor names, with the same rules {@code ContextService} reads by.
     *
     * @param projectLabel the {@code project:} half, or null when only a task was given
     * @throws UnknownAnchorException if nothing matches
     * @throws AmbiguousAnchorException if a bare task label matches in more than one project,
     *     because writing to whichever came back first is exactly the guess this design refuses
     */
    private Task resolve(String projectLabel, String taskLabel) {
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
