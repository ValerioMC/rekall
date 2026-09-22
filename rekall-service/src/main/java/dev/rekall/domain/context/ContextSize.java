package dev.rekall.domain.context;

import java.util.List;

/**
 * How much a task's context weighs when a session loads it: the characters of the markdown
 * {@code /rk} hands over, an estimate of the tokens they cost, and the parts that make them up,
 * heaviest first.
 */
public record ContextSize(int characters, int estimatedTokens, List<Part> parts) {

    /** @param reference whether the part is a note handed over as a reference rather than in full */
    public record Part(String label, int characters, boolean reference) {
    }
}
