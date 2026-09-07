package dev.rekall.bootstrap;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/api/settings/claude")
public class ClaudeCodeController {

    private static final int DEFAULT_PORT = 47355;

    private final ClaudeCodeInstaller installer;

    public ClaudeCodeController(@Value("${server.port:" + DEFAULT_PORT + "}") int port) {
        int served = port > 0 ? port : DEFAULT_PORT;
        this.installer = new ClaudeCodeInstaller("http://localhost:" + served + "/mcp");
    }

    @GetMapping
    public ClaudeCodeInstaller.Installation status() {
        return installer.status();
    }

    @PostMapping("/install")
    public ClaudeCodeInstaller.Installation install() {
        return installer.install();
    }
}
