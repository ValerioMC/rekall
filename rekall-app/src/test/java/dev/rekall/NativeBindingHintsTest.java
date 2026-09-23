package dev.rekall;

import dev.rekall.backup.BackupFile;
import dev.rekall.backup.DatabaseBackupService;
import dev.rekall.claude.queue.RunQueueView;
import dev.rekall.domain.context.ContextSize;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;
import org.springframework.aot.hint.RuntimeHints;
import org.springframework.aot.hint.annotation.ReflectiveRuntimeHintsRegistrar;
import org.springframework.aot.hint.predicate.RuntimeHintsPredicates;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * In the native image, Jackson serialising a record with a {@code List<X>} component instantiates
 * {@code X[]} reflectively, and an unregistered one answers the request with a 500. The JVM never
 * notices, so this checks that the response records carry their binding hints, arrays included.
 */
class NativeBindingHintsTest {

    @ParameterizedTest
    @ValueSource(classes = {
            ContextSize.class, ContextSize.Part[].class,
            DatabaseBackupService.Status.class, BackupFile[].class,
            RunQueueView.class, RunQueueView.Item[].class
    })
    @DisplayName("a response record and the array of its list elements are registered for binding")
    void responseTypesAreRegistered(Class<?> type) {
        RuntimeHints hints = new RuntimeHints();

        new ReflectiveRuntimeHintsRegistrar().registerRuntimeHints(hints, RekallApplication.class);

        assertThat(RuntimeHintsPredicates.reflection().onType(type)).accepts(hints);
    }
}
