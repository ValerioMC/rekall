package dev.rekall.api.controller;

import dev.rekall.api.dto.ApiDtos.TimeEntryEditRequest;
import dev.rekall.domain.timeentry.TimeEntryService;
import dev.rekall.domain.timeentry.TimeEntryView;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PatchMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.UUID;

@RestController
@RequestMapping("/api")
@RequiredArgsConstructor
public class TimeEntryController {

    private final TimeEntryService timeEntries;

    @GetMapping("/time-entries")
    public List<TimeEntryView> list() {
        return timeEntries.findAll();
    }

    @PostMapping("/tasks/{taskId}/time-entries/start")
    public TimeEntryView start(@PathVariable UUID taskId) {
        return timeEntries.start(taskId);
    }

    @PostMapping("/tasks/{taskId}/time-entries/stop")
    public TimeEntryView stop(@PathVariable UUID taskId) {
        return timeEntries.stop(taskId);
    }

    @PatchMapping("/time-entries/{id}")
    public TimeEntryView edit(@PathVariable UUID id, @Valid @RequestBody TimeEntryEditRequest request) {
        return timeEntries.edit(id, request.startedAt(), request.stoppedAt());
    }

    @DeleteMapping("/time-entries/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void delete(@PathVariable UUID id) {
        timeEntries.delete(id);
    }
}
