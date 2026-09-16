package dev.rekall.api.service;

import dev.rekall.domain.Company;
import dev.rekall.domain.Document;
import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStep;
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

@Service
@RequiredArgsConstructor
@Slf4j
public class ExportService {

    private static final int SUMMARY_LENGTH = 180;

    private final CompanyRepository companies;

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
                    String projectDir = companyDir + "/" + safe(project.getLabel());
                    zip.putNextEntry(new ZipEntry(projectDir + "/"));
                    zip.closeEntry();

                    for (Task task : project.getTasks()) {
                        String taskDir = projectDir + "/" + safe(task.getLabel());
                        zip.putNextEntry(new ZipEntry(taskDir + "/"));
                        zip.closeEntry();

                        Set<String> used = new HashSet<>();

                        if (task.getWrapup() != null) {
                            used.add("WRAPUP.md");
                            write(zip, taskDir + "/WRAPUP.md", task.getWrapup().getBodyMarkdown());
                        }

                        if (!task.getSteps().isEmpty()) {
                            used.add("STEPS.md");
                            write(zip, taskDir + "/STEPS.md", checklist(task));
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

    private String manifest(List<Company> all, Map<String, List<String>> shared) {
        StringBuilder out = new StringBuilder("# Rekall export\n\n")
                .append(LocalDate.now())
                .append("\n\nOne folder per company, then per project, then per task, one file per note.\n");

        for (Company company : all) {
            out.append("\n## ").append(company.getName())
                    .append("  `company:").append(company.getName()).append("`\n");
            if (company.getDescription() != null && !company.getDescription().isBlank()) {
                out.append("\n").append(summary(company.getDescription())).append('\n');
            }

            for (Project project : company.getProjects()) {
                out.append("\n### ").append(project.getTitle())
                        .append("  `project:").append(project.getLabel()).append("`\n\n")
                        .append("- status: ").append(project.getStatus()).append('\n');
                if (project.getDescription() != null && !project.getDescription().isBlank()) {
                    out.append("- ").append(summary(project.getDescription())).append('\n');
                }

                for (Task task : project.getTasks()) {
                    out.append("\n#### ").append(task.getTitle())
                            .append("  `project:").append(project.getLabel())
                            .append(" task:").append(task.getLabel()).append("`\n\n")
                            .append("- status: ").append(task.getStatus()).append('\n');
                    if (task.getDescription() != null && !task.getDescription().isBlank()) {
                        out.append("- ").append(summary(task.getDescription())).append('\n');
                    }
                    if (task.getWrapup() != null) {
                        out.append("- `WRAPUP.md`: the state of the implementation, written ")
                                .append(task.getWrapup().getWrittenBy() == WrapupAuthor.CLAUDE
                                        ? "by Claude" : "by hand")
                                .append('\n');
                    }
                    if (!task.getSteps().isEmpty()) {
                        long done = task.getSteps().stream().filter(TaskStep::isDone).count();
                        long checklist = task.getSteps().stream()
                                .filter(step -> !step.getState().draft()).count();
                        out.append("- `STEPS.md`: ").append(done).append(" of ")
                                .append(checklist).append(" steps done\n");
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

    private String checklist(Task task) {
        StringBuilder out = new StringBuilder("# Steps\n");
        for (TaskStep step : task.getSteps()) {
            out.append('\n').append(step.isDone() ? "- [x] " : "- [ ] ").append(step.getTitle());
            if (step.getState().draft()) {
                out.append("  (draft)");
            }
            out.append('\n');
            if (step.getBodyMarkdown() != null && !step.getBodyMarkdown().isBlank()) {
                out.append('\n').append(step.getBodyMarkdown()).append('\n');
            }
        }
        return out.toString();
    }

    private String summary(String description) {
        String flat = description.replace("\n", " ").replaceAll("\\s+", " ").trim();
        return flat.length() <= SUMMARY_LENGTH ? flat : flat.substring(0, SUMMARY_LENGTH).trim() + "\u2026";
    }

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
