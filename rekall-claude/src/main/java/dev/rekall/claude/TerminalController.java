package dev.rekall.claude;

import dev.rekall.claude.TerminalApiDtos.OpenTerminalRequest;
import dev.rekall.claude.TerminalApiDtos.TerminalView;
import dev.rekall.common.NotFoundException;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.UUID;

/**
 * Open, list and close in-app terminals. Bytes do not travel here: a pane connects to
 * {@link TerminalSocketHandler} at {@code /api/terminal/{id}/io} once the terminal exists.
 */
@RestController
@RequestMapping("/api")
@RequiredArgsConstructor
public class TerminalController {

    private final PtyTerminalManager manager;

    @GetMapping("/terminals")
    public List<TerminalView> list() {
        return manager.list();
    }

    @PostMapping("/tasks/{taskId}/terminals")
    @ResponseStatus(HttpStatus.CREATED)
    public TerminalView open(@PathVariable UUID taskId, @RequestBody(required = false) OpenTerminalRequest request) {
        OpenTerminalRequest safe = request == null
                ? new OpenTerminalRequest(null, false, null, null)
                : request;
        return manager.open(taskId, safe.stepId(), safe.skipPermissions(), safe.model(), safe.effort());
    }

    @GetMapping("/terminals/{id}")
    public TerminalView get(@PathVariable UUID id) {
        return manager.get(id).orElseThrow(() -> new NotFoundException("No terminal with id " + id));
    }

    @DeleteMapping("/terminals/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void close(@PathVariable UUID id) {
        if (manager.close(id, "Closed from the console.") == null) {
            throw new NotFoundException("No terminal with id " + id);
        }
    }
}
