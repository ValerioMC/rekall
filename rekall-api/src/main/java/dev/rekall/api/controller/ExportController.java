package dev.rekall.api.controller;

import dev.rekall.api.service.ExportService;
import lombok.RequiredArgsConstructor;
import org.springframework.http.ContentDisposition;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

/** The whole database as a zip of folders. One endpoint, no options: it exports everything. */
@RestController
@RequestMapping("/api/export")
@RequiredArgsConstructor
public class ExportController {

    private final ExportService export;

    @GetMapping
    public ResponseEntity<byte[]> download() {
        byte[] archive = export.archive();
        return ResponseEntity.ok()
                .contentType(MediaType.APPLICATION_OCTET_STREAM)
                .header(
                        HttpHeaders.CONTENT_DISPOSITION,
                        ContentDisposition.attachment().filename(export.fileNameForToday()).build().toString())
                .contentLength(archive.length)
                .body(archive);
    }
}
