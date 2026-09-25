package dev.rekall.claude;

import dev.rekall.claude.TerminalApiDtos.OpenTerminalRequest;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

/** The {@code /rk} line a terminal opens on, by what it was opened to do. */
class TerminalModeTest {

    private static final String ANCHORS = "project:vega task:report-builder";

    @Test
    @DisplayName("a work terminal loads the task")
    void workLoadsTheTask() {
        assertThat(TerminalMode.WORK.firstLine(ANCHORS)).isEqualTo("/rk project:vega task:report-builder");
    }

    @Test
    @DisplayName("a plan terminal asks the session to plan the task")
    void planPlansTheTask() {
        assertThat(TerminalMode.PLAN.firstLine(ANCHORS)).isEqualTo("/rk project:vega task:report-builder plan");
    }

    @Test
    @DisplayName("a request without a mode opens a work terminal")
    void missingModeIsWork() {
        OpenTerminalRequest request = new OpenTerminalRequest(null, false, null, null, null);

        assertThat(request.modeOrDefault()).isEqualTo(TerminalMode.WORK);
    }
}
