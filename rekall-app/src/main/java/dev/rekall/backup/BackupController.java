package dev.rekall.backup;

import lombok.RequiredArgsConstructor;
import org.springframework.core.io.FileSystemResource;
import org.springframework.core.io.Resource;
import org.springframework.http.ContentDisposition;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.multipart.MultipartFile;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.nio.file.Path;

@RestController
@RequestMapping("/api/backups")
@RequiredArgsConstructor
public class BackupController {

    private final DatabaseBackupService backups;
    private final DatabaseRestoreService restorer;

    /** A restore that was accepted: the application is restarting, and this backup holds what was there. */
    public record RestoreStarted(boolean restarting, BackupFile previousState) {
    }

    @GetMapping
    public DatabaseBackupService.Status status() {
        return backups.status();
    }

    @PostMapping
    public BackupFile backUpNow() {
        return backups.backUp(BackupFile.Reason.MANUAL);
    }

    @GetMapping("/{name}")
    public ResponseEntity<Resource> download(@PathVariable String name) {
        Path file = backups.resolve(name);
        return ResponseEntity.ok()
                .contentType(MediaType.parseMediaType("application/zip"))
                .header(HttpHeaders.CONTENT_DISPOSITION, ContentDisposition.attachment().filename(name).build().toString())
                .body(new FileSystemResource(file));
    }

    @PostMapping("/{name}/restore")
    public RestoreStarted restore(@PathVariable String name) {
        return new RestoreStarted(true, restorer.restore(name));
    }

    @PostMapping(path = "/restore", consumes = MediaType.MULTIPART_FORM_DATA_VALUE)
    public RestoreStarted restoreUpload(@RequestParam("file") MultipartFile file) {
        if (file.isEmpty()) {
            throw new IllegalArgumentException("The uploaded file is empty. Nothing was restored.");
        }
        try (InputStream upload = file.getInputStream()) {
            return new RestoreStarted(true, restorer.restoreUpload(upload));
        } catch (IOException e) {
            throw new UncheckedIOException("Could not read the uploaded file", e);
        }
    }
}
