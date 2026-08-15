package dev.rekall.bootstrap;

/** One folder that has ever held a Rekall database, registered under {@code ~/.rekall/config.json}. */
public record DatabaseEntry(String id, String label, String path, String addedAt, String lastUsedAt) {
}
