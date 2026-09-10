package dev.rekall.domain.claude;

import dev.rekall.domain.Task;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.domain.repository.TaskRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.UUID;

/**
 * Turns a task id into what the PTY needs: the folder to start {@code claude} in and the
 * {@code /rk} anchors to load. Stateless; lives in the domain layer so {@code rekall-claude} need
 * not touch a repository.
 */
@Service
@RequiredArgsConstructor
public class TerminalLaunchService {

    private final TaskRepository tasks;

    /** The folder is the project's {@code repo_folder}; a project without one is refused here. */
    @Transactional(readOnly = true)
    public TerminalLaunch resolve(UUID taskId) {
        Task task = tasks.findById(taskId)
                .orElseThrow(() -> new UnknownAnchorException("No task with id " + taskId));

        String folder = task.getProject().getRepoFolder();
        if (folder == null || folder.isBlank()) {
            throw new IllegalArgumentException(
                    "Set this project's folder on its page before opening a terminal here.");
        }

        String anchors = "project:%s task:%s".formatted(task.getProject().getLabel(), task.getLabel());
        return new TerminalLaunch(
                taskId, anchors, folder.strip(),
                task.getProject().getLabel(), task.getLabel(), task.getTitle());
    }

    public record TerminalLaunch(
            UUID taskId,
            String anchors,
            String workingDir,
            String projectLabel,
            String taskLabel,
            String taskTitle) {
    }
}
