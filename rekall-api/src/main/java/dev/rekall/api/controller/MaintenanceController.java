package dev.rekall.api.controller;

import dev.rekall.content.folder.FolderExporter;
import dev.rekall.content.folder.FolderImporter;
import dev.rekall.content.folder.ImportReport;
import dev.rekall.content.folder.ImportRequest;
import jakarta.validation.constraints.NotBlank;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.nio.file.Path;

/** One-off operations: ingesting an existing folder tree, and writing everything back out. */
@RestController
@RequestMapping("/api/maintenance")
public class MaintenanceController {

    private final FolderImporter importer;
    private final FolderExporter exporter;

    public MaintenanceController(FolderImporter importer, FolderExporter exporter) {
        this.importer = importer;
        this.exporter = exporter;
    }

    @PostMapping("/import")
    public ImportReport importFolder(@RequestBody ImportFolderRequest request) {
        ImportRequest importRequest = ImportRequest.defaults(Path.of(request.path()));
        if (request.projectEntity() != null) {
            importRequest = new ImportRequest(
                    importRequest.root(),
                    request.projectEntity(),
                    request.taskEntity() == null ? importRequest.taskEntity() : request.taskEntity(),
                    importRequest.nameField(),
                    request.taskSubfolder() == null ? importRequest.taskSubfolder() : request.taskSubfolder(),
                    importRequest.taskProjectReferenceField());
        }
        return importer.importTree(importRequest);
    }

    @PostMapping("/export")
    public ExportResponse exportFolder(@RequestBody ExportFolderRequest request) {
        return new ExportResponse(exporter.exportTo(Path.of(request.path())));
    }

    public record ImportFolderRequest(
            @NotBlank String path, String projectEntity, String taskEntity, String taskSubfolder) {
    }

    public record ExportFolderRequest(@NotBlank String path) {
    }

    public record ExportResponse(int filesWritten) {
    }
}
