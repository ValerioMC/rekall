package dev.rekall.bootstrap;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * Whether Claude Code can reach this instance, and one call to make it so.
 *
 * <p>Alongside {@link SettingsController} rather than in {@code rekall-api} for the same reason:
 * this describes the installation, not the data, and has to answer even when no database is
 * reachable. Registering Rekall with Claude Code is in fact the more useful of the two things to
 * be able to do from a broken state.
 */
@RestController
@RequestMapping("/api/settings/claude")
public class ClaudeCodeController {

    private static final int DEFAULT_PORT = 47355;

    private final ClaudeCodeInstaller installer;

    /**
     * The endpoint registered is the one this instance is actually serving, so an override of
     * {@code SERVER_PORT} registers what it started rather than what the default would have been.
     * A configured port of 0 means the random port a test boots on, which is nothing worth
     * registering; the documented port stands in for it.
     */
    public ClaudeCodeController(@Value("${server.port:" + DEFAULT_PORT + "}") int port) {
        int served = port > 0 ? port : DEFAULT_PORT;
        this.installer = new ClaudeCodeInstaller("http://localhost:" + served + "/mcp");
    }

    @GetMapping
    public ClaudeCodeInstaller.Installation status() {
        return installer.status();
    }

    /** Rewrites the registration from scratch, whatever state it was in. */
    @PostMapping("/install")
    public ClaudeCodeInstaller.Installation install() {
        return installer.install();
    }
}
