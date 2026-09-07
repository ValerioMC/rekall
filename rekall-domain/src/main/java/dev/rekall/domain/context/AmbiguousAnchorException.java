package dev.rekall.domain.context;

import java.util.List;

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
