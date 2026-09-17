package dev.rekall.domain.commit;

import org.springframework.stereotype.Component;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

/**
 * Read-only questions about a folder: is it a git repository, which branch is checked out, who
 * would git commit as. The identity is what {@code git config} resolves inside that folder, so a
 * repo-local override wins over the global one the way it does at the command line.
 */
@Component
public class GitRepositoryInspector {

    public boolean isRepository(String folder) {
        return inspect(folder).repository();
    }

    public RepositoryStatus inspect(String folder) {
        if (folder == null || folder.isBlank()) {
            return RepositoryStatus.unset();
        }
        String path = folder.strip();
        Path repoFolder = Path.of(path);
        if (!Files.isDirectory(repoFolder)) {
            return RepositoryStatus.missing(path);
        }
        GitCommand.Result inside;
        try {
            inside = GitCommand.run(repoFolder, List.of("rev-parse", "--is-inside-work-tree"));
        } catch (IllegalArgumentException gitUnavailable) {
            return RepositoryStatus.notARepository(path);
        }
        if (!inside.ok() || !"true".equals(inside.output())) {
            return RepositoryStatus.notARepository(path);
        }
        return new RepositoryStatus(
                path,
                true,
                true,
                branchOf(repoFolder),
                config(repoFolder, "user.name"),
                config(repoFolder, "user.email"));
    }

    private static String branchOf(Path repoFolder) {
        GitCommand.Result result = GitCommand.run(repoFolder, List.of("symbolic-ref", "--short", "-q", "HEAD"));
        return result.ok() && !result.output().isBlank() ? result.output() : null;
    }

    private static String config(Path repoFolder, String key) {
        GitCommand.Result result = GitCommand.run(repoFolder, List.of("config", "--get", key));
        return result.ok() && !result.output().isBlank() ? result.output() : null;
    }
}
