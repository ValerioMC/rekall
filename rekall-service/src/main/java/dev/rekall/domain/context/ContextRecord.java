package dev.rekall.domain.context;

import dev.rekall.domain.step.TaskStepView;
import dev.rekall.domain.wrapup.WrapupView;

import java.util.List;
import java.util.Map;

public record ContextRecord(
        String kind,
        String label,
        String anchor,
        Map<String, String> fields,
        List<ContextRecord> references,
        List<String> related,
        List<DocumentView> documents,
        List<TaskStepView> steps,
        WrapupView wrapup,
        String blueprint,
        String description) {

    public ContextRecord {
        fields = fields == null ? Map.of() : Map.copyOf(fields);
        references = references == null ? List.of() : List.copyOf(references);
        related = related == null ? List.of() : List.copyOf(related);
        documents = documents == null ? List.of() : List.copyOf(documents);
        steps = steps == null ? List.of() : List.copyOf(steps);
    }
}
