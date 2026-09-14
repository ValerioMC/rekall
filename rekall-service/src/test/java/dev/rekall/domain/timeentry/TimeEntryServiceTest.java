package dev.rekall.domain.timeentry;

import dev.rekall.domain.Task;
import dev.rekall.domain.TimeEntry;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TimeEntryRepository;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.time.Instant;
import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class TimeEntryServiceTest {

    private final TaskRepository tasks = mock(TaskRepository.class);
    private final TimeEntryRepository timeEntries = mock(TimeEntryRepository.class);
    private final TimeEntryService service = new TimeEntryService(tasks, timeEntries);

    @Test
    @DisplayName("shutdown stops every open timer so none keeps accruing time across a restart")
    void stopsEveryRunningTimerOnShutdown() {
        TimeEntry first = runningEntrySince(Instant.now().minusSeconds(120));
        TimeEntry second = runningEntrySince(Instant.now().minusSeconds(30));
        when(timeEntries.findAllByStoppedAtIsNull()).thenReturn(List.of(first, second));

        service.stopAllOnShutdown();

        assertThat(first.getStoppedAt()).isNotNull();
        assertThat(second.getStoppedAt()).isNotNull();
    }

    private TimeEntry runningEntrySince(Instant startedAt) {
        return new TimeEntry(mock(Task.class), startedAt);
    }
}
