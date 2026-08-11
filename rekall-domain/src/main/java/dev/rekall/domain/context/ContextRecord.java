package dev.rekall.domain.context;

import java.util.List;
import java.util.Map;

/**
 * One record and everything attached to it, fully materialised.
 *
 * <p>Materialised on purpose: it is assembled inside a read-only transaction and rendered
 * outside one. Handing entities to the renderer instead would make every field access a bet on
 * whether the session is still open.
 *
 * @param anchor the {@code entity:value} form that loads this record again
 * @param references what this record points at, each carrying its own documents. Following
 *     these is bounded by the model: a task reaches an environment and stops
 * @param related what points back at this record, as anchors only. Unbounded fan-out, so
 *     labels rather than bodies
 */
public record ContextRecord(
        String kind,
        String label,
        String anchor,
        Map<String, String> fields,
        List<ContextRecord> references,
        List<String> related,
        List<DocumentView> documents) {

    public ContextRecord {
        fields = fields == null ? Map.of() : Map.copyOf(fields);
        references = references == null ? List.of() : List.copyOf(references);
        related = related == null ? List.of() : List.copyOf(related);
        documents = documents == null ? List.of() : List.copyOf(documents);
    }
}
