package dev.rekall.domain.timeentry;

import dev.rekall.domain.Task;
import dev.rekall.domain.TimeEntry;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TimeEntryRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.Instant;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * Opens, closes and corrects the sessions a task's work is tracked in.
 *
 * <p>Only one session across the whole application may be open at a time: starting a task's
 * timer while another is running closes the other one first, the same way switching what you
 * are looking at is a single-selection thing everywhere else in this console. That rule is
 * enforced here rather than at the database, because there is exactly one writer — the console
 * — and no MCP path in, unlike the wrapup.
 */
@Service
@RequiredArgsConstructor
public class TimeEntryService {

    private final TaskRepository tasks;
    private final TimeEntryRepository timeEntries;

    /** What starting produced: the session now open, and whatever it had to close to open it. */
    public record Started(TimeEntryView started, TimeEntryView stoppedElsewhere) {
    }

    // ------------------------------------------------------------------ reading

    @Transactional(readOnly = true)
    public List<TimeEntryView> findAll() {
        return timeEntries.findAllByOrderByStartedAtDesc().stream().map(TimeEntryView::of).toList();
    }

    // ------------------------------------------------------------------ tracking

    /**
     * Opens a session on this task.
     *
     * <p>Calling it again while this same task is already the one running is a no-op, so a
     * doubled click or a race on the button does not open a second session. Calling it while a
     * different task is running closes that one first and reports it back, so the console can
     * update both rows without a refetch.
     */
    @Transactional
    public Started start(UUID taskId) {
        Task task = tasks.findById(taskId)
                .orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));

        Optional<TimeEntry> running = timeEntries.findByStoppedAtIsNull();
        if (running.isPresent() && running.get().getTask().getId().equals(taskId)) {
            return new Started(TimeEntryView.of(running.get()), null);
        }

        TimeEntryView stoppedElsewhere = null;
        if (running.isPresent()) {
            TimeEntry other = running.get();
            other.setStoppedAt(Instant.now());
            stoppedElsewhere = TimeEntryView.of(other);
        }

        TimeEntry created = new TimeEntry(task, Instant.now());
        return new Started(TimeEntryView.of(timeEntries.saveAndFlush(created)), stoppedElsewhere);
    }

    /** Closes the session open on this task. */
    @Transactional
    public TimeEntryView stop(UUID taskId) {
        tasks.findById(taskId).orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));

        TimeEntry running = timeEntries.findByStoppedAtIsNull()
                .filter(entry -> entry.getTask().getId().equals(taskId))
                .orElseThrow(() -> new IllegalArgumentException("Nothing is being tracked on this task."));
        running.setStoppedAt(Instant.now());
        return TimeEntryView.of(running);
    }

    // ------------------------------------------------------------------ correcting

    /**
     * Corrects a session by hand, for the normal case: a timer left running, or a start time
     * that was not quite when the work actually began.
     *
     * <p>A closed session cannot be reopened this way — {@code stoppedAt} may go from present to
     * absent only by staying absent, never the other direction — because "still running" is a
     * fact about the one open row, not a value any row can be edited back into. Passing a stop
     * time for the currently open session is allowed and simply closes it, so this doubles as a
     * manual alternative to {@link #stop(UUID)}.
     */
    @Transactional
    public TimeEntryView edit(UUID id, Instant startedAt, Instant stoppedAt) {
        TimeEntry entry = timeEntries.findById(id)
                .orElseThrow(() -> new UnknownAnchorException("No time entry with id " + id));

        if (startedAt == null) {
            throw new IllegalArgumentException("A session needs a start time.");
        }
        if (stoppedAt == null && entry.getStoppedAt() != null) {
            throw new IllegalArgumentException(
                    "A finished session can't be reopened. Delete it and start a new one instead.");
        }
        if (stoppedAt != null && !stoppedAt.isAfter(startedAt)) {
            throw new IllegalArgumentException("A session has to end after it starts.");
        }

        entry.setStartedAt(startedAt);
        entry.setStoppedAt(stoppedAt);
        return TimeEntryView.of(entry);
    }

    /** Removing one session. Silent when it is already gone: the end state is the same. */
    @Transactional
    public void delete(UUID id) {
        timeEntries.findById(id).ifPresent(timeEntries::delete);
    }
}
