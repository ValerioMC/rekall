package dev.rekall.api.service;

import dev.rekall.domain.Company;
import dev.rekall.domain.Document;
import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.WrapupAuthor;
import dev.rekall.domain.repository.CompanyRepository;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.time.LocalDate;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

/**
 * The whole database as a folder tree in a zip: {@code company/project/task/note.md}.
 *
 * <p>A backup and an escape hatch, not a sync. Nothing reads this format back in, which is the
 * point: if Rekall is ever abandoned the notes are still notes, in folders, readable with the
 * tool that opened them before this application existed.
 *
 * <p>The one thing the tree cannot represent is a note attached to several tasks. It is written
 * under each of them, so every folder is complete on its own, and {@code MANIFEST.md} records
 * which files are copies of one note so a reader knows not to edit them apart.
 *
 * <p>A task's wrapup is written as {@code WRAPUP.md} beside its notes. In capitals and first in
 * the folder, because on a tree someone opens two years from now it is the file that says what
 * the thing was.
 */
@Service
@RequiredArgsConstructor
@Slf4j
public class ExportService {

    private final CompanyRepository companies;

    /**
     * Built in memory rather than streamed. A single user's notes are a few hundred kilobytes of
     * markdown, and holding the archive means the transaction closes before the response starts:
     * streaming lazy associations out of a closed session is the failure this avoids.
     */
    @Transactional(readOnly = true)
    public byte[] archive() {
        List<Company> all = companies.findAllByOrderByNameAsc();
        Map<String, List<String>> shared = new TreeMap<>();
        ByteArrayOutputStream buffer = new ByteArrayOutputStream();

        try (ZipOutputStream zip = new ZipOutputStream(buffer, StandardCharsets.UTF_8)) {
            for (Company company : all) {
                String companyDir = safe(company.getName());
                zip.putNextEntry(new ZipEntry(companyDir + "/"));
                zip.closeEntry();

                for (Project project : company.getProjects()) {
                    // Folders are named after the label, not the title: the label is the name
                    // that is already an identifier, and it is what the anchor in the manifest
                    // says, so a reader can match a folder to a `/rk` line without guessing.
                    String projectDir = companyDir + "/" + safe(project.getLabel());
                    zip.putNextEntry(new ZipEntry(projectDir + "/"));
                    zip.closeEntry();

                    for (Task task : project.getTasks()) {
                        String taskDir = projectDir + "/" + safe(task.getLabel());
                        // Written even when empty, so a task with no notes is still a task.
                        zip.putNextEntry(new ZipEntry(taskDir + "/"));
                        zip.closeEntry();

                        Set<String> used = new HashSet<>();

                        // First in the folder and named in capitals, because it is the file to
                        // open first: what the task currently is, before the notes explaining
                        // it. Claimed in `used` as well, so a note that happens to be called
                        // WRAPUP.md is written alongside it rather than over it.
                        if (task.getWrapup() != null) {
                            used.add("WRAPUP.md");
                            write(zip, taskDir + "/WRAPUP.md", task.getWrapup().getBodyMarkdown());
                        }

                        for (Document document : task.getDocuments()) {
                            String path = taskDir + "/" + unique(used, fileName(document.getTitle()));
                            write(zip, path, document.getBodyMarkdown());
                            if (document.getTasks().size() > 1) {
                                shared.computeIfAbsent(document.getTitle(), key -> new ArrayList<>()).add(path);
                            }
                        }
                    }
                }
            }
            write(zip, "MANIFEST.md", manifest(all, shared));
        } catch (IOException e) {
            throw new UncheckedIOException("Could not build the export archive", e);
        }

        log.info("Exported {} company/companies as a {} byte archive", all.size(), buffer.size());
        return buffer.toByteArray();
    }

    public String fileNameForToday() {
        return "rekall-" + LocalDate.now() + ".zip";
    }

    // ------------------------------------------------------------------ contents

    private String manifest(List<Company> all, Map<String, List<String>> shared) {
        StringBuilder out = new StringBuilder("# Rekall export\n\n")
                .append(LocalDate.now())
                .append("\n\nOne folder per company, then per project, then per task, one file per note.\n");

        for (Company company : all) {
            out.append("\n## ").append(company.getName())
                    .append("  `company:").append(company.getName()).append("`\n");
            if (company.getDescription() != null && !company.getDescription().isBlank()) {
                out.append("\n").append(company.getDescription().replace("\n", " ")).append('\n');
            }

            for (Project project : company.getProjects()) {
                out.append("\n### ").append(project.getTitle())
                        .append("  `project:").append(project.getLabel()).append("`\n\n")
                        .append("- status: ").append(project.getStatus()).append('\n');
                if (project.getDescription() != null && !project.getDescription().isBlank()) {
                    out.append("- ").append(project.getDescription().replace("\n", " ")).append('\n');
                }

                for (Task task : project.getTasks()) {
                    out.append("\n#### ").append(task.getTitle())
                            .append("  `project:").append(project.getLabel())
                            .append(" task:").append(task.getLabel()).append("`\n\n")
                            .append("- status: ").append(task.getStatus()).append('\n');
                    if (task.getDescription() != null && !task.getDescription().isBlank()) {
                        out.append("- ").append(task.getDescription().replace("\n", " ")).append('\n');
                    }
                    if (task.getWrapup() != null) {
                        out.append("- `WRAPUP.md`: the state of the implementation, written ")
                                .append(task.getWrapup().getWrittenBy() == WrapupAuthor.CLAUDE
                                        ? "by Claude" : "by hand")
                                .append('\n');
                    }
                    if (task.getDocuments().isEmpty()) {
                        out.append("- no notes\n");
                    }
                }
            }
        }

        if (!shared.isEmpty()) {
            out.append("\n---\n\n## Notes that appear more than once\n\n")
                    .append("A note can be attached to several tasks. The tree has no way to say that, so it is ")
                    .append("written under each of them. These files are copies of one note: editing them apart ")
                    .append("is how the folder tree used to drift, and why this application exists.\n\n");
            shared.forEach((title, paths) -> {
                out.append("- **").append(title).append("**\n");
                paths.forEach(path -> out.append("  - `").append(path).append("`\n"));
            });
        }

        return out.toString();
    }

    private void write(ZipOutputStream zip, String path, String body) throws IOException {
        zip.putNextEntry(new ZipEntry(path));
        zip.write(body.getBytes(StandardCharsets.UTF_8));
        zip.closeEntry();
    }

    // ------------------------------------------------------------------ naming

    /**
     * A name a file system will accept, from a name a person typed.
     *
     * <p>Separators and dot segments are removed rather than escaped: a record called
     * {@code ../../etc} must become a folder name, never a path.
     */
    private String safe(String name) {
        String cleaned = name.trim()
                .replaceAll("[^A-Za-z0-9 ._-]", "-")
                .replaceAll("\\.{2,}", "-")
                .replaceAll("\\s+", "-")
                .replaceAll("-{2,}", "-")
                .replaceAll("^[-.]+|[-.]+$", "");
        return cleaned.isBlank() ? "untitled" : cleaned;
    }

    private String fileName(String title) {
        String base = safe(title);
        return base.toLowerCase().endsWith(".md") ? base : base + ".md";
    }

    /** Two notes on one task may share a title; the folder cannot hold two files with one name. */
    private String unique(Set<String> used, String name) {
        if (used.add(name)) {
            return name;
        }
        String stem = name.substring(0, name.length() - 3);
        for (int suffix = 2; ; suffix++) {
            String candidate = stem + "-" + suffix + ".md";
            if (used.add(candidate)) {
                return candidate;
            }
        }
    }
}
