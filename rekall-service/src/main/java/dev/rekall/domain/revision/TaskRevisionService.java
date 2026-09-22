package dev.rekall.domain.revision;

import dev.rekall.common.NotFoundException;
import dev.rekall.domain.RevisionKind;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskRevision;
import dev.rekall.domain.WrapupAuthor;
import dev.rekall.domain.repository.TaskRevisionRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * Keeps earlier versions of a task's wrapup and description, so replacing or deleting one is
 * never the end of it. It is called by whatever is about to overwrite the text, with the text
 * that is about to go, and decides whether that is worth a revision:
 *
 * <ul>
 *   <li>Blank text, or text identical to the newest revision, is not kept.</li>
 *   <li>A hand edit in the console autosaves as it is typed, so a hand edit over hand-written
 *       text keeps one revision per {@link #HAND_EDIT_WINDOW}: the version from before the
 *       editing started, not every keystroke on the way. A hand edit over a session's text is
 *       always kept, since that text was nobody's draft.</li>
 *   <li>A session's write, a deletion and a restore are discrete events and are always kept.</li>
 *   <li>Only the newest {@link #KEPT_PER_KIND} revisions of each text are kept per task.</li>
 * </ul>
 */
@Service
public class TaskRevisionService {

    static final int KEPT_PER_KIND = 30;

    static final Duration HAND_EDIT_WINDOW = Duration.ofMinutes(10);

    private final TaskRevisionRepository revisions;
    private final Clock clock;

    @Autowired
    public TaskRevisionService(TaskRevisionRepository revisions) {
        this(revisions, Clock.systemUTC());
    }

    TaskRevisionService(TaskRevisionRepository revisions, Clock clock) {
        this.revisions = revisions;
        this.clock = clock;
    }

    /**
     * @param outgoing       the text about to be replaced or deleted
     * @param outgoingAuthor who wrote it, when known
     * @param writtenAt      when it was written, when known
     * @return whether a revision was written
     */
    @Transactional
    public boolean keep(Task task, RevisionKind kind, String outgoing, WrapupAuthor outgoingAuthor,
                        Instant writtenAt, RevisionTrigger trigger) {
        if (outgoing == null || outgoing.isBlank()) {
            return false;
        }
        Optional<TaskRevision> newest = revisions.findFirstByTaskIdAndKindOrderByCreatedAtDesc(task.getId(), kind);
        if (newest.isPresent() && newest.get().getBodyMarkdown().equals(outgoing)) {
            return false;
        }
        if (trigger == RevisionTrigger.HAND_EDIT && outgoingAuthor != WrapupAuthor.CLAUDE && newest.isPresent()
                && newest.get().getCreatedAt().isAfter(Instant.now(clock).minus(HAND_EDIT_WINDOW))) {
            return false;
        }
        revisions.saveAndFlush(new TaskRevision(task, kind, outgoing, outgoingAuthor, writtenAt));
        prune(task.getId(), kind);
        return true;
    }

    @Transactional(readOnly = true)
    public List<TaskRevisionView> list(UUID taskId, RevisionKind kind) {
        return revisions.findByTaskIdAndKindOrderByCreatedAtDesc(taskId, kind).stream()
                .map(TaskRevisionView::of)
                .toList();
    }

    @Transactional(readOnly = true)
    public TaskRevisionView find(UUID taskId, UUID revisionId) {
        return revisions.findByIdAndTaskId(revisionId, taskId)
                .map(TaskRevisionView::of)
                .orElseThrow(() -> new NotFoundException("Revision", revisionId));
    }

    private void prune(UUID taskId, RevisionKind kind) {
        List<TaskRevision> all = revisions.findByTaskIdAndKindOrderByCreatedAtDesc(taskId, kind);
        if (all.size() > KEPT_PER_KIND) {
            revisions.deleteAll(all.subList(KEPT_PER_KIND, all.size()));
        }
    }
}
