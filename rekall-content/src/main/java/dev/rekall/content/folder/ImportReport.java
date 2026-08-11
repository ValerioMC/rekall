package dev.rekall.content.folder;

import java.util.List;

/**
 * Outcome of a folder import.
 *
 * <p>{@code warnings} is not decoration. An import that silently skipped half a tree looks
 * exactly like one that found nothing, and the difference matters when the tree is the only
 * copy of the notes being migrated.
 */
public record ImportReport(int projectsCreated, int tasksCreated, int documentsCreated, List<String> warnings) {

    public ImportReport {
        warnings = warnings == null ? List.of() : List.copyOf(warnings);
    }

    public boolean isClean() {
        return warnings.isEmpty();
    }
}
