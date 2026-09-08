package dev.rekall.claude;

import dev.rekall.domain.claude.ClaudeSessionService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.context.event.ApplicationReadyEvent;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Component;

/**
 * A hosted session's process is a child of this JVM, so a restart leaves rows that claim to be
 * live with nothing behind them. This closes them once the context is up, so the console shows
 * a finished transcript rather than a session it cannot reach.
 */
@Component
@RequiredArgsConstructor
@Slf4j
public class ClaudeSessionRecovery {

    private final ClaudeSessionService sessions;

    @EventListener(ApplicationReadyEvent.class)
    public void sweepOrphans() {
        int closed = sessions.recoverOrphans();
        if (closed > 0) {
            log.info("Closed {} Claude session(s) left open by a previous run", closed);
        }
    }
}
