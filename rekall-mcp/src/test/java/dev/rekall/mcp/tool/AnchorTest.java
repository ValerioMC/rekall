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
        List<Anchor> anchors = Anchor.parseAll("project:stvv task:code-validator");

        assertThat(anchors)
                .containsExactly(new Anchor("project", "stvv"), new Anchor("task", "code-validator"));
    }

    @Test
    @DisplayName("a term without a qualifier is positional")
    void positionalForm() {
        assertThat(Anchor.parseAll("stvv code-validator"))
                .containsExactly(new Anchor(null, "stvv"), new Anchor(null, "code-validator"));
    }

    @Test
    @DisplayName("the two forms mix in one request")
    void mixedForms() {
        assertThat(Anchor.parseAll("stvv task:code-validator"))
                .containsExactly(new Anchor(null, "stvv"), new Anchor("task", "code-validator"));
    }

    @Test
    @DisplayName("a quoted value keeps its spaces")
    void quotedValue() {
        assertThat(Anchor.parseAll("task:\"code validator\" project:stvv"))
                .containsExactly(
                        new Anchor("task", "code validator"), new Anchor("project", "stvv"));
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
        assertThat(Anchor.parseAll("repo:https://gitlab.example/stvv"))
                .containsExactly(new Anchor("repo", "https://gitlab.example/stvv"));
    }

    @Test
    @DisplayName("irregular spacing is not a syntax error")
    void extraWhitespace() {
        assertThat(Anchor.parseAll("  project:stvv   task:code-validator \n"))
                .containsExactly(new Anchor("project", "stvv"), new Anchor("task", "code-validator"));
    }

    @Test
    @DisplayName("an empty request is refused rather than answered with everything")
    void emptyRequest() {
        assertThatThrownBy(() -> Anchor.parseAll("   "))
                .isInstanceOf(ToolFailure.class)
                .hasMessageContaining("project:stvv");
    }

    @Test
    @DisplayName("a half-written anchor is refused rather than read as positional")
    void halfWrittenAnchor() {
        assertThatThrownBy(() -> Anchor.parseAll("project:"))
                .isInstanceOf(ToolFailure.class)
                .hasMessageContaining("not a valid anchor");
        assertThatThrownBy(() -> Anchor.parseAll(":stvv"))
                .isInstanceOf(ToolFailure.class)
                .hasMessageContaining("not a valid anchor");
    }
}
