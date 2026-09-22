package dev.rekall.domain.context;

import dev.rekall.domain.DocumentContextMode;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.UUID;

/**
 * Measures what loading a task costs a session, by rendering exactly what
 * {@code /rk project:<label> task:<label>} hands over and splitting it into the parts a person can
 * act on: the description, the steps, the wrapup, the commits chosen for the context, each note,
 * and the project around them.
 *
 * <p>The token figure is an estimate, {@link #CHARACTERS_PER_TOKEN} characters to a token, which
 * sits between English prose and markdown full of code; the model's own tokenizer is not available
 * here and the figure is for comparing tasks and notes, not for billing.
 */
@Service
@RequiredArgsConstructor
public class ContextSizeService {

    static final double CHARACTERS_PER_TOKEN = 3.5;

    private final ContextService context;
    private final ContextRenderer renderer;

    @Transactional(readOnly = true)
    public ContextSize measure(UUID taskId) {
        List<ContextRecord> records = context.forTask(taskId);
        int total = renderer.render(records).length();
        ContextRecord task = records.getLast();

        List<ContextSize.Part> parts = new ArrayList<>();
        addPart(parts, "Description", renderer.renderDescription(task.description()), false);
        addPart(parts, "Steps", renderer.renderSteps(task.steps(), task.wrapup()), false);
        addPart(parts, "Wrapup", renderer.renderWrapup(task.wrapup()), false);
        addPart(parts, "Commits", renderer.renderCommits(task.commits()), false);
        for (DocumentView document : task.documents()) {
            addPart(parts, "Note: " + document.title(), renderer.renderDocuments(List.of(document)),
                    document.contextMode() == DocumentContextMode.REFERENCE);
        }
        int measured = parts.stream().mapToInt(ContextSize.Part::characters).sum();
        int rest = total - measured;
        if (rest > 0) {
            parts.add(new ContextSize.Part("Project, blueprint and headings", rest, false));
        }

        parts.sort(Comparator.comparingInt(ContextSize.Part::characters).reversed());
        return new ContextSize(total, estimateTokens(total), List.copyOf(parts));
    }

    static int estimateTokens(int characters) {
        return (int) Math.ceil(characters / CHARACTERS_PER_TOKEN);
    }

    private static void addPart(List<ContextSize.Part> parts, String label, String rendered, boolean reference) {
        if (!rendered.isEmpty()) {
            parts.add(new ContextSize.Part(label, rendered.length(), reference));
        }
    }
}
