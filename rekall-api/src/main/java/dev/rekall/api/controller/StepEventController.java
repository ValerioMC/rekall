package dev.rekall.api.controller;

import dev.rekall.api.stream.StepEventStream;
import lombok.RequiredArgsConstructor;
import org.springframework.http.MediaType;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

/**
 * The console's live feed of checklist changes.
 *
 * <p>One long-lived {@code text/event-stream} connection per open window. Every step a session
 * moves over MCP, and every box ticked in another console, arrives here as a {@code steps}
 * event carrying the affected task's whole checklist, so the window that was open when it
 * happened is not left showing a stale animation.
 *
 * <p>Read-only from the client's side: there is nothing to POST here. The writes go through
 * {@code /api/steps} and {@code /mcp}; this only carries their results outward.
 */
@RestController
@RequestMapping("/api/steps")
@RequiredArgsConstructor
public class StepEventController {

    private final StepEventStream stream;

    @GetMapping(value = "/stream", produces = MediaType.TEXT_EVENT_STREAM_VALUE)
    public SseEmitter stream() {
        return stream.open();
    }
}
