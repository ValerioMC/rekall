package dev.rekall.claude;

import dev.rekall.api.service.NotFoundException;
import dev.rekall.claude.ClaudeApiDtos.PromptRequest;
import dev.rekall.claude.ClaudeApiDtos.StartSessionRequest;
import dev.rekall.domain.claude.ClaudeMessageView;
import dev.rekall.domain.claude.ClaudeSessionService;
import dev.rekall.domain.claude.ClaudeSessionView;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.util.List;
import java.util.UUID;

/**
 * The console's way in to hosted Claude sessions: list them, start one on a task, feed it a
 * prompt, watch it over SSE, stop it, remove it. There is no MCP counterpart.
 */
@RestController
@RequestMapping("/api")
@RequiredArgsConstructor
public class ClaudeSessionController {

    private final ClaudeSessionService sessions;
    private final ClaudeProcessManager manager;
    private final ClaudeSessionStream stream;

    @GetMapping("/claude/sessions")
    public List<ClaudeSessionView> list() {
        return sessions.findAll();
    }

    @GetMapping("/tasks/{taskId}/claude/sessions")
    public List<ClaudeSessionView> listForTask(@PathVariable UUID taskId) {
        return sessions.findByTask(taskId);
    }

    @PostMapping("/tasks/{taskId}/claude/sessions")
    @ResponseStatus(HttpStatus.CREATED)
    public ClaudeSessionView start(@PathVariable UUID taskId, @Valid @RequestBody StartSessionRequest request) {
        return manager.start(
                taskId, request.stepId(), request.skipPermissions(), request.model(), request.effort());
    }

    @GetMapping("/claude/sessions/{id}")
    public ClaudeSessionView get(@PathVariable UUID id) {
        return sessions.find(id).orElseThrow(() -> new NotFoundException("No Claude session with id " + id));
    }

    @GetMapping("/claude/sessions/{id}/messages")
    public List<ClaudeMessageView> transcript(@PathVariable UUID id) {
        return sessions.transcript(id);
    }

    @PostMapping("/claude/sessions/{id}/prompt")
    @ResponseStatus(HttpStatus.ACCEPTED)
    public ClaudeMessageView prompt(@PathVariable UUID id, @Valid @RequestBody PromptRequest request) {
        return manager.prompt(id, request.text());
    }

    @PostMapping("/claude/sessions/{id}/clear")
    public ClaudeSessionView clear(@PathVariable UUID id) {
        return manager.clear(id);
    }

    @PostMapping("/claude/sessions/{id}/stop")
    public ClaudeSessionView stop(@PathVariable UUID id) {
        ClaudeSessionView view = manager.stop(id, "Stopped from the console.");
        if (view == null) {
            throw new NotFoundException("No Claude session with id " + id);
        }
        return view;
    }

    @DeleteMapping("/claude/sessions/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void remove(@PathVariable UUID id) {
        manager.delete(id);
    }

    @GetMapping(value = "/claude/sessions/{id}/stream", produces = MediaType.TEXT_EVENT_STREAM_VALUE)
    public SseEmitter watch(@PathVariable UUID id) {
        return stream.open(id);
    }
}
