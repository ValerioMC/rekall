package dev.rekall.api.service;

import dev.rekall.api.dto.ApiDtos.DirectoryEntryResponse;
import dev.rekall.api.dto.ApiDtos.DirectoryListingResponse;
import dev.rekall.api.dto.ApiDtos.PathSegmentResponse;
import dev.rekall.common.NotFoundException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

class DirectoryListingServiceTest {

    @TempDir
    Path home;

    private DirectoryListingService service;

    @BeforeEach
    void setUp() throws IOException {
        service = new DirectoryListingService(home.toString());
        Path project = Files.createDirectories(home.resolve("project"));
        Files.createDirectories(project.resolve("src/components"));
        Files.createDirectories(project.resolve("Docs"));
        Files.writeString(project.resolve("README.md"), "#");
        Files.writeString(project.resolve("app.ts"), "");
        Files.writeString(project.resolve(".env.local"), "");
    }

    @Test
    @DisplayName("folders come first, then files, each by name ignoring case, hidden ones flagged")
    void listsFoldersFirstThenFiles() {
        DirectoryListingResponse listing = service.list(home.resolve("project").toString(), null);

        assertThat(listing.entries()).extracting(DirectoryEntryResponse::name)
                .containsExactly("Docs", "src", ".env.local", "app.ts", "README.md");
        assertThat(listing.entries()).filteredOn(DirectoryEntryResponse::hidden)
                .extracting(DirectoryEntryResponse::name).containsExactly(".env.local");
        assertThat(listing.entries().getFirst().path()).isEqualTo(home.resolve("project/Docs").toString());
        assertThat(listing.readable()).isTrue();
        assertThat(listing.truncated()).isFalse();
    }

    @Test
    @DisplayName("a typed path is taken relative to the folder the picker stands in, and normalised")
    void resolvesTypedPathAgainstBase() {
        String base = home.resolve("project").toString();

        assertThat(service.list(base, "src/components/../components").path())
                .isEqualTo(home.resolve("project/src/components").toString());
        assertThat(service.list(base, "..").path()).isEqualTo(home.toString());
    }

    @Test
    @DisplayName("~ is the home folder, and an absolute path ignores the base")
    void expandsHomeAndHonoursAbsolutePaths() {
        String base = home.resolve("project/src").toString();

        assertThat(service.list(base, "~").path()).isEqualTo(home.toString());
        assertThat(service.list(base, "~/project").path()).isEqualTo(home.resolve("project").toString());
        assertThat(service.list(base, home.resolve("project/Docs").toString()).path())
                .isEqualTo(home.resolve("project/Docs").toString());
    }

    @Test
    @DisplayName("with no base and no path, the listing is the home folder")
    void defaultsToHome() {
        DirectoryListingResponse listing = service.list(null, "  ");

        assertThat(listing.path()).isEqualTo(home.toString());
        assertThat(listing.home()).isEqualTo(home.toString());
    }

    @Test
    @DisplayName("the breadcrumb runs from the root to the folder, each crumb carrying its own path")
    void buildsSegmentsFromRoot() {
        Path folder = home.resolve("project/src");

        DirectoryListingResponse listing = service.list(folder.toString(), null);

        assertThat(listing.segments().getFirst().path()).isEqualTo(folder.getRoot().toString());
        assertThat(listing.segments().getLast()).isEqualTo(new PathSegmentResponse("src", folder.toString()));
        assertThat(listing.parent()).isEqualTo(home.resolve("project").toString());
    }

    @Test
    @DisplayName("a missing folder is not found, and a file is refused as a folder")
    void refusesMissingFoldersAndFiles() {
        String base = home.resolve("project").toString();

        assertThatThrownBy(() -> service.list(base, "nope")).isInstanceOf(NotFoundException.class);
        assertThatThrownBy(() -> service.list(base, "README.md"))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("is a file");
    }

    @Test
    @DisplayName("a folder bigger than the limit is cut after sorting and flagged")
    void truncatesHugeFolders() throws IOException {
        Path crowded = Files.createDirectories(home.resolve("crowded"));
        for (int index = 0; index <= DirectoryListingService.ENTRY_LIMIT; index++) {
            Files.createFile(crowded.resolve("file-%05d".formatted(index)));
        }

        DirectoryListingResponse listing = service.list(crowded.toString(), null);

        assertThat(listing.truncated()).isTrue();
        assertThat(listing.entries()).hasSize(DirectoryListingService.ENTRY_LIMIT);
    }
}
