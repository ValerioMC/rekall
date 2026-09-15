package dev.rekall.claude;

import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

/**
 * The console's read of the logged-in account's Claude usage. Always 200; whether the figures are
 * real is carried in {@link ClaudeUsageView#status()}, not the HTTP status. {@code ?refresh=true}
 * is the meter's "check again": it takes a new reading instead of serving the cached one.
 */
@RestController
@RequestMapping("/api/claude")
@RequiredArgsConstructor
public class ClaudeUsageController {

    private final ClaudeUsageService usage;

    @GetMapping("/usage")
    public ClaudeUsageView usage(@RequestParam(defaultValue = "false") boolean refresh) {
        return refresh ? usage.refresh() : usage.current();
    }
}
