package dev.rekall.claude;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import dev.rekall.domain.claude.ClaudeMessageRole;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;

import java.util.ArrayList;
import java.util.List;

/**
 * Turns one line of {@code claude --output-format stream-json} into transcript entries.
 *
 * <p>The wire format is the interactive UI's own event stream, which is not a stable contract:
 * anything this does not recognise is dropped, never guessed at. What it does recognise is the
 * shape that carries the conversation: assistant text and tool calls, tool results echoed back,
 * and the end-of-turn {@code result} with its stats.
 */
@Component
@Slf4j
public class ClaudeStreamMapper {

    private static final int TOOL_INPUT_LIMIT = 2_000;
    private static final int TOOL_RESULT_LIMIT = 4_000;

    // A private instance, like ClaudeCodeInstaller: the app has no shared ObjectMapper bean.
    private final ObjectMapper mapper = new ObjectMapper();

    public record Entry(ClaudeMessageRole role, String content, String toolName, String meta) {
    }

    public record Mapped(List<Entry> entries, boolean turnComplete, String cliSessionId, String model) {

        static Mapped empty() {
            return new Mapped(List.of(), false, null, null);
        }
    }

    public Mapped map(String line) {
        if (line == null || line.isBlank()) {
            return Mapped.empty();
        }
        JsonNode node;
        try {
            node = mapper.readTree(line);
        } catch (Exception parseFailure) {
            log.debug("Unparseable claude stream line: {}", abbreviate(line));
            return Mapped.empty();
        }

        return switch (node.path("type").asText("")) {
            case "system" -> mapSystem(node);
            case "assistant" -> new Mapped(mapAssistant(node), false, null, modelOf(node.path("message")));
            case "user" -> new Mapped(mapUser(node), false, null, null);
            case "result" -> new Mapped(mapResult(node), true, null, null);
            default -> Mapped.empty();
        };
    }

    private Mapped mapSystem(JsonNode node) {
        if (!"init".equals(node.path("subtype").asText(""))) {
            return Mapped.empty();
        }
        String cliSessionId = node.path("session_id").asText(null);
        return new Mapped(List.of(), false, cliSessionId, modelOf(node));
    }

    /** The model name off an {@code init} line or an assistant message, blank read as absent. */
    private String modelOf(JsonNode node) {
        String model = node.path("model").asText("");
        return model.isBlank() ? null : model;
    }

    private List<Entry> mapAssistant(JsonNode node) {
        List<Entry> entries = new ArrayList<>();
        for (JsonNode block : node.path("message").path("content")) {
            switch (block.path("type").asText("")) {
                case "text" -> {
                    String text = block.path("text").asText("");
                    if (!text.isBlank()) {
                        entries.add(new Entry(ClaudeMessageRole.ASSISTANT, text, null, null));
                    }
                }
                case "tool_use" -> {
                    String name = block.path("name").asText("tool");
                    String input = compact(block.path("input"), TOOL_INPUT_LIMIT);
                    String meta = toolMeta(block.path("id").asText(null), false);
                    entries.add(new Entry(ClaudeMessageRole.TOOL_USE, input, name, meta));
                }
                default -> {
                    // thinking, redacted_thinking, anything new: not part of the transcript.
                }
            }
        }
        return entries;
    }

    private List<Entry> mapUser(JsonNode node) {
        List<Entry> entries = new ArrayList<>();
        for (JsonNode block : node.path("message").path("content")) {
            if (!"tool_result".equals(block.path("type").asText(""))) {
                continue;
            }
            String text = flattenContent(block.path("content"));
            String meta = toolMeta(block.path("tool_use_id").asText(null), block.path("is_error").asBoolean(false));
            entries.add(new Entry(ClaudeMessageRole.TOOL_RESULT, clamp(text, TOOL_RESULT_LIMIT), null, meta));
        }
        return entries;
    }

    private List<Entry> mapResult(JsonNode node) {
        ObjectNode meta = mapper.createObjectNode();
        meta.put("subtype", node.path("subtype").asText("success"));
        meta.put("isError", node.path("is_error").asBoolean(false));
        if (node.hasNonNull("duration_ms")) {
            meta.put("durationMs", node.path("duration_ms").asLong());
        }
        if (node.hasNonNull("num_turns")) {
            meta.put("numTurns", node.path("num_turns").asInt());
        }
        if (node.hasNonNull("total_cost_usd")) {
            meta.put("costUsd", node.path("total_cost_usd").asDouble());
        }
        long totalTokens = totalTokensOf(node.path("usage"));
        if (totalTokens > 0) {
            meta.put("totalTokens", totalTokens);
        }
        String text = node.path("result").asText("");
        ClaudeMessageRole role = node.path("is_error").asBoolean(false)
                ? ClaudeMessageRole.ERROR
                : ClaudeMessageRole.RESULT;
        return List.of(new Entry(role, text.isBlank() ? null : text, null, meta.toString()));
    }

    /**
     * Every token the turn moved: fresh input, cache writes, cache reads and output, as
     * {@code claude} reports them on the {@code result} line's {@code usage}. Zero when there
     * is no usage block to read, so the entry keeps no {@code totalTokens} at all.
     */
    private long totalTokensOf(JsonNode usage) {
        if (usage == null || !usage.isObject()) {
            return 0L;
        }
        return usage.path("input_tokens").asLong()
                + usage.path("cache_creation_input_tokens").asLong()
                + usage.path("cache_read_input_tokens").asLong()
                + usage.path("output_tokens").asLong();
    }

    /**
     * The link between a tool call and the result echoed back for it, plus whether that result
     * was an error. Null when there is nothing to say, so an entry keeps a null meta.
     */
    private String toolMeta(String toolUseId, boolean error) {
        if (toolUseId == null && !error) {
            return null;
        }
        ObjectNode meta = mapper.createObjectNode();
        if (toolUseId != null) {
            meta.put("toolUseId", toolUseId);
        }
        if (error) {
            meta.put("error", true);
        }
        return meta.toString();
    }

    private String compact(JsonNode value, int limit) {
        if (value == null || value.isMissingNode() || value.isNull()) {
            return "";
        }
        return clamp(value.isValueNode() ? value.asText() : value.toString(), limit);
    }

    private String flattenContent(JsonNode content) {
        if (content.isTextual()) {
            return content.asText();
        }
        if (content.isArray()) {
            StringBuilder joined = new StringBuilder();
            for (JsonNode part : content) {
                if (part.hasNonNull("text")) {
                    joined.append(part.path("text").asText());
                } else if (part.isTextual()) {
                    joined.append(part.asText());
                }
            }
            return joined.toString();
        }
        return content.isMissingNode() || content.isNull() ? "" : content.toString();
    }

    private static String clamp(String value, int limit) {
        if (value == null) {
            return "";
        }
        return value.length() <= limit ? value : value.substring(0, limit) + "\n… (truncated)";
    }

    private static String abbreviate(String value) {
        return value.length() <= 160 ? value : value.substring(0, 160) + "…";
    }
}
