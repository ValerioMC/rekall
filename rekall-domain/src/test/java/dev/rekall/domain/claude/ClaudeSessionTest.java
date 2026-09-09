package dev.rekall.domain.claude;

import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The line a hosted session moves along, and the rule that a terminal state is terminal: once a
 * session has ended, nothing a late-arriving process event says can drag it back to live.
 */
class ClaudeSessionTest {

    private ClaudeSession newSession() {
        Task task = new Task("report-builder", "Report builder");
        task.setProject(new Project("vega", "Vega"));
        return new ClaudeSession(task, null, "project:vega task:report-builder", "/tmp/vega", false, null, null);
    }

    @Test
    @DisplayName("a session opens STARTING and live, with nothing behind it")
    void opensStarting() {
        ClaudeSession session = newSession();

        assertThat(session.getStatus()).isEqualTo(ClaudeSessionStatus.STARTING);
        assertThat(session.getStatus().live()).isTrue();
        assertThat(session.getEndedAt()).isNull();
        assertThat(session.getExitCode()).isNull();
    }

    @Test
    @DisplayName("working then ready then ended: each move is allowed while the last one was live")
    void walksTheLine() {
        ClaudeSession session = newSession();

        session.markStatus(ClaudeSessionStatus.WORKING);
        assertThat(session.getStatus()).isEqualTo(ClaudeSessionStatus.WORKING);

        session.markStatus(ClaudeSessionStatus.READY);
        assertThat(session.getStatus()).isEqualTo(ClaudeSessionStatus.READY);
        assertThat(session.getStatus().acceptsPrompt()).isTrue();

        session.end(ClaudeSessionStatus.EXITED, "Stopped from the console.", 0);
        assertThat(session.getStatus()).isEqualTo(ClaudeSessionStatus.EXITED);
        assertThat(session.getEndedAt()).isNotNull();
        assertThat(session.getExitCode()).isZero();
        assertThat(session.getDetail()).isEqualTo("Stopped from the console.");
    }

    @Test
    @DisplayName("once ended, a status change is ignored")
    void terminalIsTerminal() {
        ClaudeSession session = newSession();
        session.end(ClaudeSessionStatus.FAILED, "claude exited with code 1", 1);

        session.markStatus(ClaudeSessionStatus.READY);
        session.end(ClaudeSessionStatus.EXITED, "later", 0);

        assertThat(session.getStatus()).isEqualTo(ClaudeSessionStatus.FAILED);
        assertThat(session.getExitCode()).isEqualTo(1);
        assertThat(session.getDetail()).isEqualTo("claude exited with code 1");
    }

    @Test
    @DisplayName("retargetStep moves a live session's step, reports the change, and is inert once ended")
    void retargetStepMovesWhileLive() {
        ClaudeSession session = newSession();
        UUID stepOne = UUID.randomUUID();
        UUID stepTwo = UUID.randomUUID();

        assertThat(session.retargetStep(stepOne)).isTrue();
        assertThat(session.getStepId()).isEqualTo(stepOne);
        assertThat(session.retargetStep(stepOne)).isFalse();
        assertThat(session.retargetStep(stepTwo)).isTrue();
        assertThat(session.getStepId()).isEqualTo(stepTwo);

        session.end(ClaudeSessionStatus.EXITED, "done", 0);
        assertThat(session.retargetStep(UUID.randomUUID())).isFalse();
        assertThat(session.getStepId()).isEqualTo(stepTwo);
    }

    @Test
    @DisplayName("the cli session id is captured once and never overwritten")
    void cliSessionIdIsStickyOnce() {
        ClaudeSession session = newSession();

        session.attachCliSession(null);
        assertThat(session.getCliSessionId()).isNull();

        session.attachCliSession("cli-" + UUID.randomUUID());
        String first = session.getCliSessionId();
        assertThat(first).isNotNull();

        session.attachCliSession("cli-something-else");
        assertThat(session.getCliSessionId()).isEqualTo(first);
    }

    @Test
    @DisplayName("the model is left null when none was chosen, then filled in by what claude reports")
    void resolveModelFillsInWhatWasNotChosen() {
        ClaudeSession session = newSession();
        assertThat(session.getModel()).isNull();

        assertThat(session.resolveModel("claude-sonnet-4-5-20250929")).isTrue();
        assertThat(session.getModel()).isEqualTo("claude-sonnet-4-5-20250929");

        assertThat(session.resolveModel("claude-sonnet-4-5-20250929")).isFalse();
        assertThat(session.resolveModel(null)).isFalse();
        assertThat(session.resolveModel(" ")).isFalse();
    }

    @Test
    @DisplayName("a chosen model is replaced by the concrete id claude names for it")
    void resolveModelReplacesTheAlias() {
        Task task = new Task("report-builder", "Report builder");
        task.setProject(new Project("vega", "Vega"));
        ClaudeSession session = new ClaudeSession(
                task, null, "project:vega task:report-builder", "/tmp/vega", false, "opus", "high");
        assertThat(session.getModel()).isEqualTo("opus");
        assertThat(session.getEffort()).isEqualTo("high");

        assertThat(session.resolveModel("claude-opus-4-1-20250805")).isTrue();
        assertThat(session.getModel()).isEqualTo("claude-opus-4-1-20250805");
    }

    @Test
    @DisplayName("once ended, the model claude reports late is ignored like any other event")
    void resolveModelIgnoredAfterEnd() {
        ClaudeSession session = newSession();
        session.end(ClaudeSessionStatus.EXITED, "done", 0);

        assertThat(session.resolveModel("claude-sonnet-4-5-20250929")).isFalse();
        assertThat(session.getModel()).isNull();
    }

    @Test
    @DisplayName("end() with a live status passed in still lands on EXITED, never on a live state")
    void endNormalisesToTerminal() {
        ClaudeSession session = newSession();

        session.end(ClaudeSessionStatus.WORKING, "odd", null);

        assertThat(session.getStatus()).isEqualTo(ClaudeSessionStatus.EXITED);
        assertThat(session.getStatus().live()).isFalse();
    }
}
