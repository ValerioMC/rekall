package dev.rekall.api.controller;

import dev.rekall.domain.context.ContextSize;
import dev.rekall.domain.context.ContextSizeService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.UUID;

@RestController
@RequestMapping("/api/tasks/{taskId}/context-size")
@RequiredArgsConstructor
public class ContextSizeController {

    private final ContextSizeService sizes;

    @GetMapping
    public ContextSize measure(@PathVariable UUID taskId) {
        return sizes.measure(taskId);
    }
}
