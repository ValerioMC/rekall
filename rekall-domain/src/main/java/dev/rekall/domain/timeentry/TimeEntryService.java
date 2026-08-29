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
 * <p>Only one session on a given task may be open at a time — starting a task's timer twice is
 * a no-op, not a second row — but different tasks may each have one open at once: work happens
 * in parallel, and a timer belongs to the task it is tracking, not to a single global cursor.
 * That rule is enforced here rather than at the database, because there is exactly one writer —
 * the console — and no MCP path in, unlike the wrapup.
 */
@Service
@RequiredArgsConstructor
public class TimeEntryService {

    private final TaskRepository tasks;
    private final TimeEntryRepository timeEntries;

    // ------------------------------------------------------------------ reading

    @Transactional(readOnly = true)
    public List<TimeEntryView> findAll() {
        return timeEntries.findAllByOrderByStartedAtDesc().stream().map(TimeEntryView::of).toList();
    }

    // ------------------------------------------------------------------ tracking

    /**
     * Opens a session on this task, leaving whatever else is running on other tasks untouched.
     *
     * <p>Calling it again while this same task is already running is a no-op, so a doubled
     * click or a race on the button does not open a second session.
     */
    @Transactional
    public TimeEntryView start(UUID taskId) {
        Task task = tasks.findById(taskId)
                .orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));

        Optional<TimeEntry> running = timeEntries.findByTaskIdAndStoppedAtIsNull(taskId);
        if (running.isPresent()) {
            return TimeEntryView.of(running.get());
        }

        TimeEntry created = new TimeEntry(task, Instant.now());
        return TimeEntryView.of(timeEntries.saveAndFlush(created));
    }

    /** Closes the session open on this task. */
    @Transactional
    public TimeEntryView stop(UUID taskId) {
        tasks.findById(taskId).orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));

        TimeEntry running = timeEntries.findByTaskIdAndStoppedAtIsNull(taskId)
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
