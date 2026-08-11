package dev.rekall.content.folder;

import java.nio.file.Path;

/**
 * Where to import from, and which entities the folder levels correspond to.
 *
 * <p>The defaults describe the layout Rekall was built to replace: one directory per project,
 * an {@code issues} directory inside it, one directory per task, and markdown files at both
 * levels. Every name is a parameter because the importer must not assume the entities are
 * called what the author happened to call them.
 */
public record ImportRequest(
        Path root,
        String projectEntity,
        String taskEntity,
        String nameField,
        String taskSubfolder,
        String taskProjectReferenceField) {

    public static ImportRequest defaults(Path root) {
        return new ImportRequest(root, "project", "task", "name", "issues", "project_id");
    }
}
