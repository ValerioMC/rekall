package dev.rekall.domain.context;

import java.util.List;

/**
 * A value matched more than one record.
 *
 * <p>Carries the candidates rather than a bare message: the caller is expected to show them so
 * the next attempt is qualified, and choosing one here would be the guessing the whole design
 * exists to avoid.
 */
public class AmbiguousAnchorException extends RuntimeException {

    private final transient List<String> candidates;

    public AmbiguousAnchorException(String value, List<String> candidates) {
        super("'%s' matches %d records: %s".formatted(value, candidates.size(), String.join(", ", candidates)));
        this.candidates = List.copyOf(candidates);
    }

    public List<String> getCandidates() {
        return candidates;
    }
}
