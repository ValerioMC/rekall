package dev.rekall.domain.wrapup;

import dev.rekall.domain.Task;
import dev.rekall.domain.Wrapup;
import dev.rekall.domain.WrapupAuthor;
import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.WrapupRepository;
import dev.rekall.domain.review.TaskReviewService;
import lombok.RequiredArgsConstructor;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

@Service
@RequiredArgsConstructor
public class WrapupService {

    private final TaskRepository tasks;
    private final WrapupRepository wrapups;
    private final TaskReviewService taskReview;
    private final ApplicationEventPublisher events;

    public record Written(WrapupView wrapup, boolean created, WrapupAuthor replaced) {
    }

    @Transactional(readOnly = true)
    public Optional<WrapupView> find(UUID taskId) {
        return wrapups.findByTaskId(taskId).map(WrapupView::of);
    }

    @Transactional(readOnly = true)
    public List<WrapupView> findAll() {
        return wrapups.findAllByOrderByUpdatedAtDesc().stream().map(WrapupView::of).toList();
    }

    @Transactional(readOnly = true)
    public Optional<WrapupView> find(String projectLabel, String taskLabel) {
        return find(resolve(projectLabel, taskLabel).getId());
    }

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
            Written written = new Written(WrapupView.of(wrapups.saveAndFlush(wrapup)), true, null);
            claimIfClaude(task, author);
            events.publishEvent(WrapupStreamEvent.written(written.wrapup()));
            return written;
        }
        Wrapup wrapup = existing.get();
        WrapupAuthor previous = wrapup.getWrittenBy();
        wrapup.setBodyMarkdown(text);
        wrapup.setWrittenBy(author);
        Written written = new Written(WrapupView.of(wrapups.saveAndFlush(wrapup)), false, previous);
        claimIfClaude(task, author);
        events.publishEvent(WrapupStreamEvent.written(written.wrapup()));
        return written;
    }

    // A Claude-authored wrapup is what a stepless task delivers, so writing one is what advances
    // its review line to CLAIMED. A hand-written one is the reviewer's correction, not a claim.
    private void claimIfClaude(Task task, WrapupAuthor author) {
        if (author == WrapupAuthor.CLAUDE) {
            taskReview.claimedByWrapup(task.getId());
        }
    }

    @Transactional
    public void delete(UUID taskId) {
        wrapups.findByTaskId(taskId).ifPresent(wrapup -> {
            wrapup.getTask().setWrapup(null);
            wrapups.delete(wrapup);
            events.publishEvent(WrapupStreamEvent.deleted(taskId));
        });
    }

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
