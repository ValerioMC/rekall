package dev.rekall.domain.note;

import java.util.UUID;

/** Emitted when a session writes a new note onto a task, so an open console can pick it up. */
public record NoteStreamEvent(UUID taskId, UUID documentId) {
}
