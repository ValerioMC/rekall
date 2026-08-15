package dev.rekall.bootstrap;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.nio.file.Path;
import java.util.List;
import java.util.Optional;

import static org.assertj.core.api.Assertions.assertThat;

class DatabaseRegistryStoreTest {

    @TempDir
    Path home;

    @Test
    void aRegistryThatWasNeverWrittenReadsAsEmpty() {
        assertThat(new DatabaseRegistryStore(home).read()).isEmpty();
    }

    @Test
    void whatIsWrittenIsWhatComesBack() {
        DatabaseRegistryStore store = new DatabaseRegistryStore(home);
        DatabaseEntry entry =
                new DatabaseEntry("id-1", "Local", "/tmp/rekall", "2026-01-01T00:00:00Z", "2026-01-01T00:00:00Z");
        store.write(new DatabaseRegistry("id-1", List.of(entry)));

        Optional<DatabaseRegistry> read = store.read();

        assertThat(read).isPresent();
        assertThat(read.get().activeId()).isEqualTo("id-1");
        assertThat(read.get().active()).isEqualTo(entry);
    }

    @Test
    void writingTwiceOverwritesRatherThanAppending() {
        DatabaseRegistryStore store = new DatabaseRegistryStore(home);
        store.write(new DatabaseRegistry("a", List.of(new DatabaseEntry("a", "First", "/a", "t", "t"))));
        store.write(new DatabaseRegistry("b", List.of(new DatabaseEntry("b", "Second", "/b", "t", "t"))));

        assertThat(store.read().orElseThrow().databases()).hasSize(1);
        assertThat(store.read().orElseThrow().activeId()).isEqualTo("b");
    }

    @Test
    void theHomeDirectoryIsCreatedOnFirstWrite() {
        Path nested = home.resolve("nested/.rekall");
        DatabaseRegistryStore store = new DatabaseRegistryStore(nested);

        store.write(new DatabaseRegistry("a", List.of(new DatabaseEntry("a", "First", "/a", "t", "t"))));

        assertThat(nested.resolve("config.json")).exists();
    }

    @Test
    void anEntryWithNoMatchingActiveIdIsReportedAsNoActiveEntry() {
        DatabaseRegistryStore store = new DatabaseRegistryStore(home);
        store.write(new DatabaseRegistry("missing", List.of(new DatabaseEntry("a", "First", "/a", "t", "t"))));

        assertThat(store.read().orElseThrow().active()).isNull();
    }
}
