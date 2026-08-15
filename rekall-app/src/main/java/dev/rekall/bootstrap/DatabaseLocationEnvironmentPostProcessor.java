package dev.rekall.bootstrap;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.env.EnvironmentPostProcessor;
import org.springframework.core.env.ConfigurableEnvironment;
import org.springframework.core.env.MapPropertySource;

import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;

/**
 * Resolves where the H2 file lives before the datasource bean is built.
 *
 * <p>{@code application.yaml} reads {@code REKALL_DB_URL}, falling back to
 * {@code rekall.resolved-db-url}, which is always set here: an explicit environment variable
 * (tests, power users) still wins outright, everyone else gets whatever the registry — or the
 * legacy {@code ./data} folder, for anyone upgrading from before this existed — resolves to.
 *
 * <p>Runs again, fresh, every time {@code ApplicationRestarter} reboots the context in place,
 * which is what makes a Settings change take effect without the process itself being relaunched.
 */
public class DatabaseLocationEnvironmentPostProcessor implements EnvironmentPostProcessor {

    private static final Path LEGACY_FOLDER = Path.of("./data");
    private static final String BOOTSTRAP_URL = "jdbc:h2:mem:rekall-setup;DB_CLOSE_DELAY=-1";

    @Override
    public void postProcessEnvironment(ConfigurableEnvironment environment, SpringApplication application) {
        DatabaseRegistryStore store = new DatabaseRegistryStore();
        Optional<DatabaseRegistry> registry = store.read();

        String url;
        String status;

        if (registry.isPresent() && reachable(registry.get().active())) {
            url = urlFor(registry.get().active().path());
            status = "READY";
        } else if (registry.isEmpty() && legacyDataFolderExists()) {
            DatabaseRegistry adopted = adoptLegacyFolder();
            store.write(adopted);
            url = urlFor(adopted.active().path());
            status = "READY";
        } else if (registry.isPresent()) {
            // A folder was configured once but is not reachable right now (unplugged drive,
            // renamed cloud folder). Booting into a blank setup wizard here would read as data
            // loss, so this is reported as its own status instead of SETUP_NEEDED.
            url = BOOTSTRAP_URL;
            status = "UNREACHABLE";
        } else {
            url = BOOTSTRAP_URL;
            status = "SETUP_NEEDED";
        }

        Map<String, Object> resolved = new LinkedHashMap<>();
        resolved.put("rekall.resolved-db-url", url);
        resolved.put("rekall.setup.status", status);
        environment.getPropertySources().addLast(new MapPropertySource("rekallDatabaseLocation", resolved));
    }

    private boolean reachable(DatabaseEntry entry) {
        return entry != null && Files.isDirectory(Path.of(entry.path()));
    }

    private boolean legacyDataFolderExists() {
        return Files.isRegularFile(LEGACY_FOLDER.resolve("rekall.mv.db"));
    }

    private DatabaseRegistry adoptLegacyFolder() {
        String id = UUID.randomUUID().toString();
        String path = LEGACY_FOLDER.toAbsolutePath().normalize().toString();
        String now = Instant.now().toString();
        return new DatabaseRegistry(id, List.of(new DatabaseEntry(id, "Local", path, now, now)));
    }

    private String urlFor(String path) {
        return "jdbc:h2:file:" + path + "/rekall;AUTO_SERVER=TRUE;DB_CLOSE_DELAY=-1";
    }
}
