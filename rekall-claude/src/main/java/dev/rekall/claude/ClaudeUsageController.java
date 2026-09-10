package dev.rekall.claude;

import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * The console's read of the logged-in account's Claude usage. Always 200; whether the figures are
 * real is carried in {@link ClaudeUsageView#status()}, not the HTTP status.
 */
@RestController
@RequestMapping("/api/claude")
@RequiredArgsConstructor
public class ClaudeUsageController {

    private final ClaudeUsageService usage;

    @GetMapping("/usage")
    public ClaudeUsageView usage() {
        return usage.current();
    }
}
