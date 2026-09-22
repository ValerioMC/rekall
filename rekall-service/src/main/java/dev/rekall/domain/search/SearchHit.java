package dev.rekall.domain.search;

import java.util.UUID;

/**
 * One place a search term was found: which text ({@link Kind}), the record that holds it, where
 * that record lives, and the words around the match.
 *
 * @param taskId     the task to open: the one the text is on, or for a note the first task it is on
 * @param stepId     the step, for a {@link Kind#STEP} hit
 * @param documentId the note, for a {@link Kind#NOTE} hit
 * @param title      what the record is called: the task's, the step's or the note's title
 * @param where      the anchor of the task the text is on, for a note the one it opens on
 * @param excerpt    the match with some words either side, on one line
 */
public record SearchHit(
        Kind kind, UUID taskId, UUID stepId, UUID documentId, String title, String where, String excerpt) {

    public enum Kind {
        DESCRIPTION,
        STEP,
        WRAPUP,
        NOTE
    }
}
