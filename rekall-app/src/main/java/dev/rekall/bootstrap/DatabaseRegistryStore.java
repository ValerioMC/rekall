package dev.rekall.bootstrap;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Optional;

/**
 * Where the database lives cannot be recorded inside the database itself, so it is a small JSON
 * file next to it.
 *
 * <p>The home directory defaults to {@code ~/.rekall} but is read from the {@code rekall.home}
 * system property first, so tests can point it at a build-local temp directory and never touch a
 * real machine's configuration. Built with a plain {@link ObjectMapper} rather than an injected
 * one: this class also runs from {@link DatabaseLocationEnvironmentPostProcessor}, before the
 * Spring context (and its configured {@code ObjectMapper} bean) exists.
 */
public class DatabaseRegistryStore {

    private final Path configFile;
    private final ObjectMapper mapper = new ObjectMapper().enable(SerializationFeature.INDENT_OUTPUT);

    public DatabaseRegistryStore() {
        this(Path.of(System.getProperty("rekall.home", System.getProperty("user.home") + "/.rekall")));
    }

    public DatabaseRegistryStore(Path home) {
        this.configFile = home.resolve("config.json");
    }

    public Optional<DatabaseRegistry> read() {
        if (!Files.isRegularFile(configFile)) {
            return Optional.empty();
        }
        try {
            return Optional.of(mapper.readValue(configFile.toFile(), DatabaseRegistry.class));
        } catch (IOException e) {
            throw new UncheckedIOException("Could not read " + configFile, e);
        }
    }

    public void write(DatabaseRegistry registry) {
        try {
            Files.createDirectories(configFile.getParent());
            mapper.writeValue(configFile.toFile(), registry);
        } catch (IOException e) {
            throw new UncheckedIOException("Could not write " + configFile, e);
        }
    }
}
