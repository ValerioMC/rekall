package dev.rekall.bootstrap;

import java.util.List;

/**
 * Every database folder Rekall knows about, and which one is live.
 *
 * <p>Switching is changing {@code activeId}; nothing here ever deletes a folder or the database
 * inside it, so "forgetting" an entry only removes it from this list.
 */
public record DatabaseRegistry(String activeId, List<DatabaseEntry> databases) {

    public DatabaseEntry active() {
        return databases.stream().filter(entry -> entry.id().equals(activeId)).findFirst().orElse(null);
    }
}
