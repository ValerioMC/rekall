package dev.rekall.content.folder;

import dev.rekall.engine.data.DynamicRecordRepository;
import dev.rekall.engine.data.QueryFilter;
import dev.rekall.engine.data.RecordView;
import dev.rekall.engine.schema.SchemaRegistry;
import dev.rekall.meta.domain.Document;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.repository.DocumentRepository;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

/**
 * Writes every document back out as a folder tree.
 *
 * <p>The escape hatch for the decision to keep markdown in Postgres. It exists so that
 * choosing the database as the source of truth never means the content is locked in there:
 * one command and the notes are ordinary files again.
 */
@Service
@RequiredArgsConstructor
@Slf4j
public class FolderExporter {

    private final SchemaRegistry registry;
    private final DynamicRecordRepository records;
    private final DocumentRepository documents;

    /** @return number of files written */
    @Transactional(readOnly = true)
    public int exportTo(Path root) {
        int written = 0;
        for (MetaTable entity : registry.entities()) {
            Map<UUID, String> labels = labelsOf(entity);
            List<Document> entityDocuments = documents.findByEntityNameOrderByPositionAsc(entity.getPhysicalName());

            for (Document document : entityDocuments) {
                String label = labels.getOrDefault(document.getRecordId(), document.getRecordId().toString());
                Path target = root.resolve(entity.getPhysicalName())
                        .resolve(sanitise(label))
                        .resolve(fileName(document.getTitle()));
                write(target, document.getBodyMarkdown());
                written++;
            }
        }
        log.info("Exported {} document(s) to {}", written, root);
        return written;
    }

    private Map<UUID, String> labelsOf(MetaTable entity) {
        Map<UUID, String> labels = new HashMap<>();
        records.query(entity, new QueryFilter(List.of(), List.of(), QueryFilter.MAX_LIMIT, 0), 0)
                .forEach(view -> labels.put(view.id(), view.label()));
        return labels;
    }

    private void write(Path target, String body) {
        try {
            Files.createDirectories(target.getParent());
            Files.writeString(target, body);
        } catch (IOException e) {
            throw new UncheckedIOException("Could not write " + target, e);
        }
    }

    /**
     * Keeps a label usable as a path segment. Titles come from imported file names and can
     * contain separators, which would otherwise write outside the intended directory.
     */
    private String sanitise(String value) {
        String cleaned = value.replaceAll("[/\\\\:*?\"<>|]", "-").trim();
        if (cleaned.isEmpty() || cleaned.equals(".") || cleaned.equals("..")) {
            return "untitled";
        }
        return cleaned;
    }

    /** Titles imported from a tree already end in {@code .md}; ones typed in the UI may not. */
    private String fileName(String title) {
        String cleaned = sanitise(title);
        return cleaned.toLowerCase(java.util.Locale.ROOT).endsWith(".md") ? cleaned : cleaned + ".md";
    }
}
