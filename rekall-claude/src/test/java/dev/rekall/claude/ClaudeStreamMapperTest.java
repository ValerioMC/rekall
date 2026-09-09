package dev.rekall.claude;

import dev.rekall.claude.ClaudeStreamMapper.Mapped;
import dev.rekall.domain.claude.ClaudeMessageRole;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The one-line-at-a-time translation of {@code claude --output-format stream-json} into
 * transcript entries. Lines it does not know are dropped, not guessed.
 */
class ClaudeStreamMapperTest {

    private final ClaudeStreamMapper mapper = new ClaudeStreamMapper();

    @Test
    @DisplayName("the init line carries the cli session id and the model, and adds nothing to the transcript")
    void initLine() {
        Mapped mapped = mapper.map(
                "{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"abc-123\","
                        + "\"model\":\"claude-sonnet-4-5-20250929\"}");

        assertThat(mapped.cliSessionId()).isEqualTo("abc-123");
        assertThat(mapped.model()).isEqualTo("claude-sonnet-4-5-20250929");
        assertThat(mapped.entries()).isEmpty();
        assertThat(mapped.turnComplete()).isFalse();
    }

    @Test
    @DisplayName("an init line with no model reports none")
    void initLineWithoutModel() {
        Mapped mapped = mapper.map("{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"abc-123\"}");

        assertThat(mapped.model()).isNull();
    }

    @Test
    @DisplayName("an assistant line reports the model it names, so a mid-session switch is caught")
    void assistantLineCarriesTheModel() {
        String line = "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\","
                + "\"model\":\"claude-opus-4-1-20250805\",\"content\":["
                + "{\"type\":\"text\",\"text\":\"On it.\"}]}}";

        assertThat(mapper.map(line).model()).isEqualTo("claude-opus-4-1-20250805");
    }

    @Test
    @DisplayName("an assistant line becomes one entry per text block and one per tool call")
    void assistantLine() {
        String line = "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":["
                + "{\"type\":\"text\",\"text\":\"Looking at the file.\"},"
                + "{\"type\":\"tool_use\",\"name\":\"Read\",\"input\":{\"file_path\":\"/a/b.txt\"}}]}}";

        Mapped mapped = mapper.map(line);

        assertThat(mapped.entries()).hasSize(2);
        assertThat(mapped.entries().get(0).role()).isEqualTo(ClaudeMessageRole.ASSISTANT);
        assertThat(mapped.entries().get(0).content()).isEqualTo("Looking at the file.");
        assertThat(mapped.entries().get(1).role()).isEqualTo(ClaudeMessageRole.TOOL_USE);
        assertThat(mapped.entries().get(1).toolName()).isEqualTo("Read");
        assertThat(mapped.entries().get(1).content()).contains("/a/b.txt");
        assertThat(mapped.turnComplete()).isFalse();
    }

    @Test
    @DisplayName("a tool call carries its id in meta so a result can be paired with it")
    void toolCallCarriesItsId() {
        String line = "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":["
                + "{\"type\":\"tool_use\",\"id\":\"tu_1\",\"name\":\"Bash\",\"input\":{\"command\":\"git status\"}}]}}";

        Mapped mapped = mapper.map(line);

        assertThat(mapped.entries()).hasSize(1);
        assertThat(mapped.entries().getFirst().role()).isEqualTo(ClaudeMessageRole.TOOL_USE);
        assertThat(mapped.entries().getFirst().meta()).contains("\"toolUseId\":\"tu_1\"");
    }

    @Test
    @DisplayName("a tool_result echoed on a user line becomes a TOOL_RESULT entry paired by id")
    void toolResultLine() {
        String line = "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":["
                + "{\"type\":\"tool_result\",\"tool_use_id\":\"tu_1\",\"content\":[{\"type\":\"text\",\"text\":\"line one\"}]}]}}";

        Mapped mapped = mapper.map(line);

        assertThat(mapped.entries()).hasSize(1);
        assertThat(mapped.entries().getFirst().role()).isEqualTo(ClaudeMessageRole.TOOL_RESULT);
        assertThat(mapped.entries().getFirst().content()).isEqualTo("line one");
        assertThat(mapped.entries().getFirst().meta()).contains("\"toolUseId\":\"tu_1\"");
    }

    @Test
    @DisplayName("a tool_result flagged is_error carries that flag in meta")
    void erroredToolResult() {
        String line = "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":["
                + "{\"type\":\"tool_result\",\"tool_use_id\":\"tu_9\",\"is_error\":true,"
                + "\"content\":[{\"type\":\"text\",\"text\":\"no such file\"}]}]}}";

        Mapped mapped = mapper.map(line);

        assertThat(mapped.entries()).hasSize(1);
        assertThat(mapped.entries().getFirst().role()).isEqualTo(ClaudeMessageRole.TOOL_RESULT);
        assertThat(mapped.entries().getFirst().meta())
                .contains("\"toolUseId\":\"tu_9\"")
                .contains("\"error\":true");
    }

    @Test
    @DisplayName("plain user text is our own echo and is dropped")
    void plainUserTextIsDropped() {
        String line = "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":["
                + "{\"type\":\"text\",\"text\":\"hello\"}]}}";

        assertThat(mapper.map(line).entries()).isEmpty();
    }

    @Test
    @DisplayName("the result line completes the turn and carries its stats in meta")
    void resultLine() {
        String line = "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,"
                + "\"result\":\"done\",\"duration_ms\":1234,\"num_turns\":3,\"total_cost_usd\":0.0125,"
                + "\"usage\":{\"input_tokens\":12,\"cache_creation_input_tokens\":300,"
                + "\"cache_read_input_tokens\":8000,\"output_tokens\":88}}";

        Mapped mapped = mapper.map(line);

        assertThat(mapped.turnComplete()).isTrue();
        assertThat(mapped.entries()).hasSize(1);
        assertThat(mapped.entries().getFirst().role()).isEqualTo(ClaudeMessageRole.RESULT);
        assertThat(mapped.entries().getFirst().meta())
                .contains("\"durationMs\":1234")
                .contains("\"numTurns\":3")
                .contains("\"costUsd\":0.0125")
                .contains("\"totalTokens\":8400");
    }

    @Test
    @DisplayName("a result line with no usage block carries no token count")
    void resultLineWithoutUsage() {
        String line = "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,"
                + "\"result\":\"done\",\"num_turns\":1}";

        assertThat(mapper.map(line).entries().getFirst().meta())
                .doesNotContain("totalTokens");
    }

    @Test
    @DisplayName("a result flagged is_error is an ERROR entry, still ending the turn")
    void erroredResult() {
        String line = "{\"type\":\"result\",\"subtype\":\"error_during_execution\",\"is_error\":true,"
                + "\"result\":\"boom\"}";

        Mapped mapped = mapper.map(line);

        assertThat(mapped.turnComplete()).isTrue();
        assertThat(mapped.entries().getFirst().role()).isEqualTo(ClaudeMessageRole.ERROR);
        assertThat(mapped.entries().getFirst().content()).isEqualTo("boom");
    }

    @Test
    @DisplayName("junk and unknown line types are silently ignored")
    void junkIsIgnored() {
        assertThat(mapper.map("not json").entries()).isEmpty();
        assertThat(mapper.map("").entries()).isEmpty();
        assertThat(mapper.map("{\"type\":\"stream_event\",\"delta\":{}}").entries()).isEmpty();
    }
}
