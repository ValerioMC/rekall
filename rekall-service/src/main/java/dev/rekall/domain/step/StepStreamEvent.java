package dev.rekall.domain.step;

import java.util.List;
import java.util.UUID;

public record StepStreamEvent(UUID taskId, List<TaskStepView> steps) {

    public StepStreamEvent {
        steps = steps == null ? List.of() : List.copyOf(steps);
    }
}
