package dev.rekall.content.folder;

import dev.rekall.content.DocumentService;
import dev.rekall.engine.data.DynamicRecordRepository;
import dev.rekall.engine.data.QueryFilter;
import dev.rekall.engine.data.RecordView;
import dev.rekall.engine.schema.SchemaRegistry;
import dev.rekall.meta.domain.MetaTable;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.Stream;

/**
 * Ingests an existing folder tree of markdown notes into the meta-model.
 *
 * <p>One direction only, and not idempotent by design beyond matching records that already
 * carry the same name: this is a migration you run once, not a sync. A watcher that tried to
 * keep a directory and a database agreeing would need conflict resolution, and the database is
 * the source of truth precisely so that question never arises.
 */
@Service
public class FolderImporter {

    private static final Logger log = LoggerFactory.getLogger(FolderImporter.class);

    private final SchemaRegistry registry;
    private final DynamicRecordRepository records;
    private final DocumentService documents;

    public FolderImporter(SchemaRegistry registry, DynamicRecordRepository records, DocumentService documents) {
        this.registry = registry;
        this.records = records;
        this.documents = documents;
    }

    @Transactional
    public ImportReport importTree(ImportRequest request) {
        List<String> warnings = new ArrayList<>();

        if (!Files.isDirectory(request.root())) {
            return new ImportReport(0, 0, 0, List.of("Not a directory: " + request.root()));
        }
        Optional<MetaTable> projectEntity = registry.entity(request.projectEntity());
        if (projectEntity.isEmpty()) {
            return new ImportReport(
                    0, 0, 0, List.of("Entity '%s' is not defined yet".formatted(request.projectEntity())));
        }
        Optional<MetaTable> taskEntity = registry.entity(request.taskEntity());
        if (taskEntity.isEmpty()) {
            warnings.add("Entity '%s' is not defined: task folders will be skipped".formatted(request.taskEntity()));
        }

        Counters counters = new Counters();
        for (Path projectDir : subdirectories(request.root())) {
            importProject(projectDir, request, projectEntity.get(), taskEntity, counters, warnings);
        }

        log.info(
                "Imported {} project(s), {} task(s), {} document(s) from {}",
                counters.projects,
                counters.tasks,
                counters.documents,
                request.root());
        return new ImportReport(counters.projects, counters.tasks, counters.documents, warnings);
    }

    private void importProject(
            Path projectDir,
            ImportRequest request,
            MetaTable projectEntity,
            Optional<MetaTable> taskEntity,
            Counters counters,
            List<String> warnings) {

        String projectName = projectDir.getFileName().toString();
        UUID projectId = findOrCreate(projectEntity, request.nameField(), projectName, Map.of(), counters::countProject);

        Path taskRoot = projectDir.resolve(request.taskSubfolder());
        attachMarkdown(projectDir, projectEntity.getPhysicalName(), projectId, taskRoot, counters, warnings);

        if (!Files.isDirectory(taskRoot) || taskEntity.isEmpty()) {
            return;
        }
        for (Path taskDir : subdirectories(taskRoot)) {
            String taskName = taskDir.getFileName().toString();
            Map<String, Object> extra = referenceToProject(taskEntity.get(), request, projectId, warnings);
            UUID taskId = findOrCreate(taskEntity.get(), request.nameField(), taskName, extra, counters::countTask);
            attachMarkdown(taskDir, taskEntity.get().getPhysicalName(), taskId, null, counters, warnings);
        }
    }

    /**
     * Links the task to its project only if the reference field actually exists, so a
     * meta-model without that link still imports rather than failing outright.
     */
    private Map<String, Object> referenceToProject(
            MetaTable taskEntity, ImportRequest request, UUID projectId, List<String> warnings) {
        if (taskEntity.findField(request.taskProjectReferenceField()).isEmpty()) {
            warnings.add("'%s' has no field '%s': imported tasks will not be linked to their project"
                    .formatted(taskEntity.getPhysicalName(), request.taskProjectReferenceField()));
            return Map.of();
        }
        return Map.of(request.taskProjectReferenceField(), projectId);
    }

    private UUID findOrCreate(
            MetaTable entity, String nameField, String name, Map<String, Object> extra, Runnable onCreate) {

        if (entity.findField(nameField).isEmpty()) {
            throw new IllegalStateException(
                    "'%s' has no field '%s' to match records by".formatted(entity.getPhysicalName(), nameField));
        }
        List<RecordView> existing =
                records.query(entity, QueryFilter.where(QueryFilter.Condition.eq(nameField, name)), 0);
        if (!existing.isEmpty()) {
            return existing.getFirst().id();
        }
        Map<String, Object> values = new HashMap<>(extra);
        values.put(nameField, name);
        onCreate.run();
        return records.insert(entity, values);
    }

    /** Attaches every markdown file under {@code dir}, skipping the subtree given by {@code exclude}. */
    private void attachMarkdown(
            Path dir, String entityName, UUID recordId, Path exclude, Counters counters, List<String> warnings) {

        try (Stream<Path> walk = Files.walk(dir)) {
            List<Path> markdown = walk.filter(Files::isRegularFile)
                    .filter(path -> path.getFileName().toString().toLowerCase(Locale.ROOT).endsWith(".md"))
                    .filter(path -> exclude == null || !path.startsWith(exclude))
                    .sorted(Comparator.comparing(Path::toString))
                    .toList();

            for (Path file : markdown) {
                try {
                    String title = dir.relativize(file).toString();
                    documents.create(entityName, recordId, title, kindOf(file), Files.readString(file));
                    counters.documents++;
                } catch (IOException e) {
                    warnings.add("Could not read " + file + ": " + e.getMessage());
                }
            }
        } catch (IOException e) {
            throw new UncheckedIOException("Could not walk " + dir, e);
        }
    }

    /**
     * Derives a classification from the file name. {@code CONTEXT.md} is the convention the
     * existing tree already uses for the file that orients you in a task.
     */
    private String kindOf(Path file) {
        String name = file.getFileName().toString().toLowerCase(Locale.ROOT);
        if (name.startsWith("context")) {
            return "context";
        }
        if (name.contains("architettura") || name.contains("architecture")) {
            return "architecture";
        }
        if (name.contains("report")) {
            return "report";
        }
        return "notes";
    }

    private List<Path> subdirectories(Path root) {
        try (Stream<Path> children = Files.list(root)) {
            return children.filter(Files::isDirectory)
                    .filter(path -> !path.getFileName().toString().startsWith("."))
                    .sorted(Comparator.comparing(Path::toString))
                    .toList();
        } catch (IOException e) {
            throw new UncheckedIOException("Could not list " + root, e);
        }
    }

    private static final class Counters {
        private int projects;
        private int tasks;
        private int documents;

        void countProject() {
            projects++;
        }

        void countTask() {
            tasks++;
        }
    }
}
