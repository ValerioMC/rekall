package dev.rekall.mcp.tool;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

/**
 * The anchor syntax is the whole user-facing surface of {@code /rk}, and it is the one place
 * where a silent misparse sends the wrong record into a context window.
 */
class AnchorTest {

    @Test
    @DisplayName("the canonical form splits into entity and value")
    void qualifiedForm() {
        List<Anchor> anchors = Anchor.parseAll("project:vega task:report-builder");

        assertThat(anchors)
                .containsExactly(new Anchor("project", "vega"), new Anchor("task", "report-builder"));
    }

    @Test
    @DisplayName("a term without a qualifier is positional")
    void positionalForm() {
        assertThat(Anchor.parseAll("vega report-builder"))
                .containsExactly(new Anchor(null, "vega"), new Anchor(null, "report-builder"));
    }

    @Test
    @DisplayName("the two forms mix in one request")
    void mixedForms() {
        assertThat(Anchor.parseAll("vega task:report-builder"))
                .containsExactly(new Anchor(null, "vega"), new Anchor("task", "report-builder"));
    }

    @Test
    @DisplayName("a quoted value keeps its spaces")
    void quotedValue() {
        assertThat(Anchor.parseAll("task:\"report builder\" project:vega"))
                .containsExactly(
                        new Anchor("task", "report builder"), new Anchor("project", "vega"));
    }

    @Test
    @DisplayName("a colon inside a quoted value belongs to the value")
    void colonInsideQuotes() {
        assertThat(Anchor.parseAll("\"ESA-4412: main workflow\""))
                .containsExactly(new Anchor(null, "ESA-4412: main workflow"));
    }

    @Test
    @DisplayName("only the first colon qualifies, so a url survives as a value")
    void onlyTheFirstColonQualifies() {
        assertThat(Anchor.parseAll("repo:https://gitlab.example/vega"))
                .containsExactly(new Anchor("repo", "https://gitlab.example/vega"));
    }

    @Test
    @DisplayName("irregular spacing is not a syntax error")
    void extraWhitespace() {
        assertThat(Anchor.parseAll("  project:vega   task:report-builder \n"))
                .containsExactly(new Anchor("project", "vega"), new Anchor("task", "report-builder"));
    }

    @Test
    @DisplayName("an empty request is refused rather than answered with everything")
    void emptyRequest() {
        assertThatThrownBy(() -> Anchor.parseAll("   "))
                .isInstanceOf(ToolFailure.class)
                .hasMessageContaining("project:vega");
    }

    @Test
    @DisplayName("a half-written anchor is refused rather than read as positional")
    void halfWrittenAnchor() {
        assertThatThrownBy(() -> Anchor.parseAll("project:"))
                .isInstanceOf(ToolFailure.class)
                .hasMessageContaining("not a valid anchor");
        assertThatThrownBy(() -> Anchor.parseAll(":vega"))
                .isInstanceOf(ToolFailure.class)
                .hasMessageContaining("not a valid anchor");
    }
}
