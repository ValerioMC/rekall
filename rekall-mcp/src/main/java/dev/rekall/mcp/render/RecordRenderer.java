package dev.rekall.mcp.render;

import dev.rekall.engine.data.RecordView;
import dev.rekall.mcp.ReadOnlyDataAccess;
import org.springframework.stereotype.Component;

import java.util.Arrays;
import java.util.List;
import java.util.Map;

/**
 * Renders records as markdown rather than JSON.
 *
 * <p>The consumer is a language model reading a context window, not a parser. Markdown costs
 * fewer tokens for the same information and keeps a resolved reference visually nested under
 * the field that pointed at it, which is exactly the relationship the answer needs to express.
 */
@Component
public class RecordRenderer {

    /** Bodies longer than this are cut, with the cut announced rather than silent. */
    private static final int MAX_DOCUMENT_CHARACTERS = 20_000;

    public String renderRecord(RecordView view, int indentLevel) {
        String indent = "  ".repeat(indentLevel);
        StringBuilder out = new StringBuilder();
        out.append(indent).append("### ").append(view.label()).append('\n');
        out.append(indent).append("- `id`: ").append(view.id()).append('\n');

        for (Map.Entry<String, Object> entry : view.values().entrySet()) {
            Object value = entry.getValue();
            if (value == null) {
                continue;
            }
            if (value instanceof RecordView nested) {
                out.append(indent).append("- `").append(entry.getKey()).append("` -> ").append(nested.label())
                        .append('\n');
                for (Map.Entry<String, Object> nestedEntry : nested.values().entrySet()) {
                    if (nestedEntry.getValue() == null || nestedEntry.getValue() instanceof RecordView) {
                        continue;
                    }
                    out.append(indent).append("    - `").append(nestedEntry.getKey()).append("`: ")
                            .append(scalar(nestedEntry.getValue())).append('\n');
                }
                continue;
            }
            out.append(indent).append("- `").append(entry.getKey()).append("`: ").append(scalar(value)).append('\n');
        }
        if (view.updatedAt() != null) {
            out.append(indent).append("- `updated_at`: ").append(view.updatedAt()).append('\n');
        }
        return out.toString();
    }

    /** Full document bodies, for the tools whose job is to put a task into context. */
    public String renderDocuments(List<ReadOnlyDataAccess.DocumentSummary> documents) {
        if (documents.isEmpty()) {
            return "";
        }
        StringBuilder out = new StringBuilder("\n#### Documents\n");
        for (ReadOnlyDataAccess.DocumentSummary document : documents) {
            out.append("\n<document title=\"").append(document.title()).append("\" kind=\"")
                    .append(document.kind()).append("\">\n")
                    .append(truncate(document.bodyMarkdown()))
                    .append("\n</document>\n");
        }
        return out.toString();
    }

    private String truncate(String body) {
        if (body == null) {
            return "";
        }
        if (body.length() <= MAX_DOCUMENT_CHARACTERS) {
            return body;
        }
        return body.substring(0, MAX_DOCUMENT_CHARACTERS)
                + "\n\n[truncated: %d of %d characters shown. Ask for this document specifically to see the rest.]"
                        .formatted(MAX_DOCUMENT_CHARACTERS, body.length());
    }

    private String scalar(Object value) {
        if (value instanceof String[] array) {
            return String.join(", ", Arrays.asList(array));
        }
        return String.valueOf(value);
    }
}
