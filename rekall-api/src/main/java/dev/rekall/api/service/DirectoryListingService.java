package dev.rekall.api.service;

import dev.rekall.api.dto.ApiDtos.DirectoryEntryResponse;
import dev.rekall.api.dto.ApiDtos.DirectoryListingResponse;
import dev.rekall.api.dto.ApiDtos.PathSegmentResponse;
import dev.rekall.common.NotFoundException;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import java.io.IOException;
import java.nio.file.AccessDeniedException;
import java.nio.file.DirectoryStream;
import java.nio.file.Files;
import java.nio.file.InvalidPathException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * Lists one folder on this machine for the path picker in the markdown editor.
 *
 * <p>It reads names only, never contents, and it lives in rekall-api so it stays off the MCP
 * classpath: browsing the disk is something the console does for the person typing, not
 * something a session is handed. The server is loopback-only ({@code LocalAccessFilter}), so
 * the reach is the reach of the person already sitting at the machine.
 *
 * <p>A path is resolved the way a shell would: {@code ~} is the home folder, an absolute path is
 * itself, and anything else is taken relative to the folder the picker is standing in. That is
 * what lets someone type {@code src/comp} or {@code ~/Downl} into the picker's filter and land
 * in the right place.
 */
@Slf4j
@Service
public class DirectoryListingService {

    /**
     * Enough for any folder a person browses by eye; a folder bigger than this (a cache, a
     * node_modules) is cut off after sorting and flagged; a folder past the cut is still reached by typing its path.
     */
    static final int ENTRY_LIMIT = 2000;

    private static final Comparator<DirectoryEntryResponse> FOLDERS_FIRST_THEN_NAME =
            Comparator.comparing((DirectoryEntryResponse entry) -> !entry.directory())
                    .thenComparing(DirectoryEntryResponse::name, String.CASE_INSENSITIVE_ORDER);

    private final Path home;

    public DirectoryListingService(@Value("${user.home}") String home) {
        this.home = Path.of(home).toAbsolutePath().normalize();
    }

    public DirectoryListingResponse list(String base, String path) {
        Path folder = resolve(base, path);
        if (!Files.exists(folder)) {
            throw new NotFoundException("There is no folder at " + folder + ".");
        }
        if (!Files.isDirectory(folder)) {
            throw new IllegalArgumentException(folder + " is a file, not a folder.");
        }
        List<DirectoryEntryResponse> entries = new ArrayList<>();
        boolean readable = readEntries(folder, entries);
        entries.sort(FOLDERS_FIRST_THEN_NAME);
        boolean truncated = entries.size() > ENTRY_LIMIT;
        List<DirectoryEntryResponse> shown = truncated ? List.copyOf(entries.subList(0, ENTRY_LIMIT)) : entries;
        Path parent = folder.getParent();
        return new DirectoryListingResponse(
                folder.toString(),
                parent == null ? null : parent.toString(),
                home.toString(),
                segments(folder),
                shown,
                readable,
                truncated);
    }

    Path resolve(String base, String path) {
        Path start = blank(base) ? home : expandHome(base.trim());
        if (blank(path)) {
            return start.toAbsolutePath().normalize();
        }
        Path typed = expandHome(path.trim());
        return start.resolve(typed).toAbsolutePath().normalize();
    }

    private Path expandHome(String raw) {
        try {
            if (raw.equals("~")) {
                return home;
            }
            if (raw.startsWith("~/") || raw.startsWith("~\\")) {
                return home.resolve(raw.substring(2));
            }
            return Path.of(raw);
        } catch (InvalidPathException e) {
            throw new IllegalArgumentException("\"" + raw + "\" is not a path this machine understands.", e);
        }
    }

    /** False when the folder exists but would not be listed; the entries are left empty. */
    private boolean readEntries(Path folder, List<DirectoryEntryResponse> into) {
        try (DirectoryStream<Path> children = Files.newDirectoryStream(folder)) {
            for (Path child : children) {
                into.add(entry(child));
            }
            return true;
        } catch (AccessDeniedException e) {
            log.info("Folder listing refused: {}", folder);
            into.clear();
            return false;
        } catch (IOException e) {
            log.warn("Folder listing failed: {}", folder, e);
            into.clear();
            return false;
        }
    }

    private DirectoryEntryResponse entry(Path child) {
        String name = child.getFileName().toString();
        return new DirectoryEntryResponse(name, child.toString(), Files.isDirectory(child), isHidden(child, name));
    }

    private static boolean isHidden(Path child, String name) {
        if (name.startsWith(".")) {
            return true;
        }
        try {
            return Files.isHidden(child);
        } catch (IOException e) {
            return false;
        }
    }

    private static List<PathSegmentResponse> segments(Path folder) {
        List<PathSegmentResponse> segments = new ArrayList<>();
        Path root = folder.getRoot();
        Path walked = root;
        if (root != null) {
            segments.add(new PathSegmentResponse(root.toString(), root.toString()));
        }
        for (Path name : folder) {
            walked = walked == null ? name : walked.resolve(name);
            segments.add(new PathSegmentResponse(name.toString(), walked.toString()));
        }
        return segments;
    }

    private static boolean blank(String value) {
        return value == null || value.isBlank();
    }
}
