package dev.rekall.bootstrap;

import java.util.List;

public record DatabaseRegistry(String activeId, List<DatabaseEntry> databases) {

    public DatabaseEntry active() {
        return databases.stream().filter(entry -> entry.id().equals(activeId)).findFirst().orElse(null);
    }
}
