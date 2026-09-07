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

@Service
@RequiredArgsConstructor
public class TimeEntryService {

    private final TaskRepository tasks;
    private final TimeEntryRepository timeEntries;

    @Transactional(readOnly = true)
    public List<TimeEntryView> findAll() {
        return timeEntries.findAllByOrderByStartedAtDesc().stream().map(TimeEntryView::of).toList();
    }

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

    @Transactional
    public TimeEntryView stop(UUID taskId) {
        tasks.findById(taskId).orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));

        TimeEntry running = timeEntries.findByTaskIdAndStoppedAtIsNull(taskId)
                .orElseThrow(() -> new IllegalArgumentException("Nothing is being tracked on this task."));
        running.setStoppedAt(Instant.now());
        return TimeEntryView.of(running);
    }

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

    @Transactional
    public void delete(UUID id) {
        timeEntries.findById(id).ifPresent(timeEntries::delete);
    }
}
