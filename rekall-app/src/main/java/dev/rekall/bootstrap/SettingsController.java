package dev.rekall.bootstrap;

import dev.rekall.ApplicationRestarter;
import dev.rekall.api.service.ConflictException;
import dev.rekall.api.service.NotFoundException;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PatchMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

import java.nio.file.Path;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * Where the database is, and every folder it has ever been.
 *
 * <p>Deliberately outside {@code rekall-api}: this has no dependency on a repository or an
 * entity, because it has to keep working in the {@code SETUP_NEEDED} and {@code UNREACHABLE}
 * states, when the real datasource is a throwaway in-memory database or missing entirely. Every
 * mutating endpoint here writes {@code ~/.rekall/config.json} and then restarts the application
 * so the next boot picks up the change; the entry it returns describes what {@code will} be true
 * after that restart completes.
 */
@RestController
@RequestMapping("/api/settings/databases")
public class SettingsController {

    private final DatabaseRegistryStore store = new DatabaseRegistryStore();

    public record DatabaseView(
            String id, String label, String path, boolean active, boolean reachable, String addedAt, String lastUsedAt) {
    }

    public record StatusResponse(String status, DatabaseView active, List<DatabaseView> databases) {
    }

    public record AddRequest(String path, String label) {
    }

    public record AddResponse(String mode, DatabaseView entry) {
    }

    public record RenameRequest(String label) {
    }

    public record CheckResponse(
            String resolvedPath, boolean exists, boolean isDirectory, boolean writable, boolean hasDatabase, boolean usable) {
    }

    @GetMapping
    public StatusResponse status() {
        Optional<DatabaseRegistry> maybeRegistry = store.read();
        if (maybeRegistry.isEmpty()) {
            return new StatusResponse("SETUP_NEEDED", null, List.of());
        }
        DatabaseRegistry registry = maybeRegistry.get();
        List<DatabaseView> views = registry.databases().stream().map(entry -> view(entry, registry)).toList();
        DatabaseView active = views.stream().filter(DatabaseView::active).findFirst().orElse(null);
        String status = active == null ? "SETUP_NEEDED" : active.reachable() ? "READY" : "UNREACHABLE";
        return new StatusResponse(status, active, views);
    }

    /** Read-only, so the folder field can show what will happen before anything commits to it. */
    @GetMapping("/check")
    public CheckResponse check(@RequestParam String path) {
        FolderValidation.Result result = FolderValidation.inspect(path);
        return new CheckResponse(
                result.resolvedPath(), result.exists(), result.isDirectory(), result.writable(),
                result.hasDatabase(), result.usable());
    }

    /** Registers a folder and switches to it, opening the database already there or creating one. */
    @PostMapping
    @ResponseStatus(HttpStatus.CREATED)
    public AddResponse add(@RequestBody AddRequest request) {
        FolderValidation.Result check = FolderValidation.inspect(request.path());
        if (!check.exists() || !check.isDirectory()) {
            throw new IllegalArgumentException(
                    "The folder \"" + check.resolvedPath() + "\" does not exist. Create it, then try again.");
        }
        if (!check.writable()) {
            throw new IllegalArgumentException("The folder \"" + check.resolvedPath() + "\" is not writable.");
        }

        DatabaseRegistry current = store.read().orElse(new DatabaseRegistry(null, List.of()));
        String id = UUID.randomUUID().toString();
        String now = Instant.now().toString();
        String label = request.label() == null || request.label().isBlank()
                ? defaultLabel(check.resolvedPath())
                : request.label().trim();
        DatabaseEntry entry = new DatabaseEntry(id, label, check.resolvedPath(), now, now);

        List<DatabaseEntry> databases = new ArrayList<>(current.databases());
        databases.add(entry);
        DatabaseRegistry updated = new DatabaseRegistry(id, databases);
        store.write(updated);

        ApplicationRestarter.restart();
        return new AddResponse(check.hasDatabase() ? "opened" : "created", view(entry, updated));
    }

    /** Switches to an already-registered folder. Refuses a folder that is no longer reachable. */
    @PostMapping("/{id}/activate")
    public DatabaseView activate(@PathVariable String id) {
        DatabaseRegistry current = store.read().orElseThrow(() -> new NotFoundException("No database is configured yet"));
        DatabaseEntry entry = find(current, id);
        FolderValidation.Result check = FolderValidation.inspect(entry.path());
        if (!check.isDirectory()) {
            throw new IllegalArgumentException(
                    "\"" + entry.label() + "\" points at " + entry.path() + ", which is not reachable right now.");
        }

        List<DatabaseEntry> updated = current.databases().stream()
                .map(candidate -> candidate.id().equals(id) ? withLastUsed(candidate) : candidate)
                .toList();
        DatabaseRegistry registry = new DatabaseRegistry(id, updated);
        store.write(registry);

        ApplicationRestarter.restart();
        return view(find(registry, id), registry);
    }

    /** Renaming never restarts anything: the active database does not change. */
    @PatchMapping("/{id}")
    public DatabaseView rename(@PathVariable String id, @RequestBody RenameRequest request) {
        if (request.label() == null || request.label().isBlank()) {
            throw new IllegalArgumentException("A database needs a name.");
        }
        DatabaseRegistry current = store.read().orElseThrow(() -> new NotFoundException("No database is configured yet"));
        find(current, id);

        List<DatabaseEntry> updated = current.databases().stream()
                .map(candidate -> candidate.id().equals(id) ? renamed(candidate, request.label().trim()) : candidate)
                .toList();
        DatabaseRegistry registry = new DatabaseRegistry(current.activeId(), updated);
        store.write(registry);
        return view(find(registry, id), registry);
    }

    /** Forgets the entry. Never touches the files on disk, and refuses the one currently active. */
    @DeleteMapping("/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void forget(@PathVariable String id) {
        DatabaseRegistry current = store.read().orElseThrow(() -> new NotFoundException("No database is configured yet"));
        find(current, id);
        if (id.equals(current.activeId())) {
            throw new ConflictException("This is the database currently in use. Switch to another one before forgetting it.");
        }
        List<DatabaseEntry> remaining = current.databases().stream().filter(entry -> !entry.id().equals(id)).toList();
        store.write(new DatabaseRegistry(current.activeId(), remaining));
    }

    // ------------------------------------------------------------------ helpers

    private DatabaseEntry find(DatabaseRegistry registry, String id) {
        return registry.databases().stream().filter(entry -> entry.id().equals(id)).findFirst()
                .orElseThrow(() -> new NotFoundException("No database registered with id " + id));
    }

    private DatabaseEntry withLastUsed(DatabaseEntry entry) {
        return new DatabaseEntry(entry.id(), entry.label(), entry.path(), entry.addedAt(), Instant.now().toString());
    }

    private DatabaseEntry renamed(DatabaseEntry entry, String label) {
        return new DatabaseEntry(entry.id(), label, entry.path(), entry.addedAt(), entry.lastUsedAt());
    }

    private String defaultLabel(String resolvedPath) {
        Path fileName = Path.of(resolvedPath).getFileName();
        return fileName == null ? "Database" : fileName.toString();
    }

    private DatabaseView view(DatabaseEntry entry, DatabaseRegistry registry) {
        boolean reachable = FolderValidation.inspect(entry.path()).isDirectory();
        return new DatabaseView(
                entry.id(), entry.label(), entry.path(), entry.id().equals(registry.activeId()), reachable,
                entry.addedAt(), entry.lastUsedAt());
    }
}
