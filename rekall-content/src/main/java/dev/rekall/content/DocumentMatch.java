package dev.rekall.content;

import java.util.UUID;

/**
 * A document that matched a search, with the record it belongs to.
 *
 * @param excerpt the matching passage with the terms marked, so an answer can quote the hit
 *     instead of pulling the whole document into context
 */
public record DocumentMatch(
        UUID documentId,
        String entityName,
        UUID recordId,
        String title,
        String kind,
        String excerpt,
        double rank) {
}
