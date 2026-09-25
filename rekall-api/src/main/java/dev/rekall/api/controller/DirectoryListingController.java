package dev.rekall.api.controller;

import dev.rekall.api.dto.ApiDtos.DirectoryListingResponse;
import dev.rekall.api.service.DirectoryListingService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

/**
 * {@code base} is the folder the picker stands in and {@code path} what was typed on top of it;
 * both are optional, and with neither the listing is the home folder.
 */
@RestController
@RequestMapping("/api/filesystem/directory")
@RequiredArgsConstructor
public class DirectoryListingController {

    private final DirectoryListingService listings;

    @GetMapping
    public DirectoryListingResponse list(
            @RequestParam(required = false) String base, @RequestParam(required = false) String path) {
        return listings.list(base, path);
    }
}
