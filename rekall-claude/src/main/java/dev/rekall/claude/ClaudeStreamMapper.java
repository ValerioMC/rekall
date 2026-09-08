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

    public record Mapped(List<Entry> entries, boolean turnComplete, String cliSessionId) {

        static Mapped empty() {
            return new Mapped(List.of(), false, null);
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
            case "assistant" -> new Mapped(mapAssistant(node), false, null);
            case "user" -> new Mapped(mapUser(node), false, null);
            case "result" -> new Mapped(mapResult(node), true, null);
            default -> Mapped.empty();
        };
    }

    private Mapped mapSystem(JsonNode node) {
        if (!"init".equals(node.path("subtype").asText(""))) {
            return Mapped.empty();
        }
        String cliSessionId = node.path("session_id").asText(null);
        return new Mapped(List.of(), false, cliSessionId);
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
                    entries.add(new Entry(ClaudeMessageRole.TOOL_USE, input, name, null));
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
            String meta = null;
            String toolUseId = block.path("tool_use_id").asText(null);
            if (toolUseId != null) {
                meta = "{\"toolUseId\":\"" + toolUseId.replace("\"", "'") + "\"}";
            }
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
        String text = node.path("result").asText("");
        ClaudeMessageRole role = node.path("is_error").asBoolean(false)
                ? ClaudeMessageRole.ERROR
                : ClaudeMessageRole.RESULT;
        return List.of(new Entry(role, text.isBlank() ? null : text, null, meta.toString()));
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
