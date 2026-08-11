package dev.rekall.engine.execute;

import java.util.List;
import java.util.UUID;

/** What an applied plan actually did. */
public record DdlApplyResult(UUID planId, List<String> statements, int documentsDeleted) {

    public int statementCount() {
        return statements.size();
    }
}
