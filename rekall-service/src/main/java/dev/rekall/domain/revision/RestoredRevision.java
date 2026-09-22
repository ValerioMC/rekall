package dev.rekall.domain.revision;

import dev.rekall.domain.RevisionKind;

import java.util.UUID;

/** What a restore wrote back: which text of which task, and its new current body. */
public record RestoredRevision(UUID taskId, RevisionKind kind, String bodyMarkdown) {
}
