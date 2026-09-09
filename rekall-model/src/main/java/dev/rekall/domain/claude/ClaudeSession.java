package dev.rekall.domain.claude;

import dev.rekall.domain.Task;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.FetchType;
import jakarta.persistence.ForeignKey;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.Table;
import lombok.Getter;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/**
 * One run of Claude Code, hosted inside Rekall.
 *
 * <p>The row is created {@code STARTING} and walked along {@link ClaudeSessionStatus} by
 * {@code ClaudeProcessManager} as the process reports in. {@code stepId} says which step the
 * session was opened against, with no foreign key: the process manager moves that step to
 * {@code RUNNING} on start and releases it on end, and nothing else reads it.
 */
@Entity
@Table(name = "claude_session")
@Getter
public class ClaudeSession {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(
            name = "task_id",
            nullable = false,
            updatable = false,
            foreignKey = @ForeignKey(name = "fk_claude_session_task"))
    private Task task;

    @Column(name = "step_id")
    private UUID stepId;

    @Column(name = "anchors", nullable = false, length = 300)
    private String anchors;

    @Column(name = "working_dir", nullable = false, length = 1_000)
    private String workingDir;

    @Column(name = "cli_session_id", length = 200)
    private String cliSessionId;

    @Column(name = "model", length = 60)
    private String model;

    @Column(name = "effort", length = 10)
    private String effort;

    @Enumerated(EnumType.STRING)
    @Column(name = "status", nullable = false, length = 20)
    private ClaudeSessionStatus status = ClaudeSessionStatus.STARTING;

    @Column(name = "skip_permissions", nullable = false)
    private boolean skipPermissions;

    @Column(name = "detail", length = 2_000)
    private String detail;

    @Column(name = "exit_code")
    private Integer exitCode;

    @Column(name = "last_activity_at", nullable = false)
    private Instant lastActivityAt = Instant.now();

    @Column(name = "started_at", nullable = false)
    private Instant startedAt = Instant.now();

    @Column(name = "ended_at")
    private Instant endedAt;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected ClaudeSession() {
    }

    public ClaudeSession(
            Task task, UUID stepId, String anchors, String workingDir, boolean skipPermissions,
            String model, String effort) {
        this.task = task;
        this.stepId = stepId;
        this.anchors = anchors;
        this.workingDir = workingDir;
        this.skipPermissions = skipPermissions;
        this.model = model == null || model.isBlank() ? null : model.strip();
        this.effort = effort == null || effort.isBlank() ? null : effort.strip();
    }

    public void touch() {
        this.lastActivityAt = Instant.now();
    }

    public void attachCliSession(String id) {
        if (id != null && !id.isBlank() && this.cliSessionId == null) {
            this.cliSessionId = id;
        }
    }

    /**
     * Record the model {@code claude} says it is running, once its {@code system}/{@code init}
     * line names it. Unlike {@link #attachCliSession}, this may replace an earlier value: the
     * session is created holding the alias it was asked for, and the concrete id is better.
     * Returns whether anything changed, so the caller can decide to announce it.
     */
    public boolean resolveModel(String resolved) {
        if (resolved == null || resolved.isBlank() || resolved.equals(this.model) || !this.status.live()) {
            return false;
        }
        this.model = resolved.strip();
        touch();
        return true;
    }

    /**
     * Point a live session at a different step, because "Run here" was pressed on one while this
     * session was already up and it is the process that will drive it. Ignored once the session
     * has ended. Returns whether the target actually moved.
     */
    public boolean retargetStep(UUID nextStepId) {
        if (!this.status.live() || Objects.equals(this.stepId, nextStepId)) {
            return false;
        }
        this.stepId = nextStepId;
        touch();
        return true;
    }

    public void markStatus(ClaudeSessionStatus next) {
        if (next == null || this.status == next || !this.status.live()) {
            return;
        }
        this.status = next;
        touch();
    }

    public void end(ClaudeSessionStatus terminal, String detail, Integer exitCode) {
        if (!this.status.live()) {
            return;
        }
        this.status = terminal == null || terminal.live() ? ClaudeSessionStatus.EXITED : terminal;
        this.detail = detail;
        this.exitCode = exitCode;
        this.endedAt = Instant.now();
        touch();
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof ClaudeSession that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "ClaudeSession[" + anchors + ", " + status + "]";
    }
}
