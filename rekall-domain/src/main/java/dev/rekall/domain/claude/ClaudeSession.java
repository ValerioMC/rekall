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
 * {@code ClaudeProcessManager} as the process reports in. {@code stepId} is a soft hint with no
 * foreign key: it says which step the session was opened against and nothing more.
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

    public ClaudeSession(Task task, UUID stepId, String anchors, String workingDir, boolean skipPermissions) {
        this.task = task;
        this.stepId = stepId;
        this.anchors = anchors;
        this.workingDir = workingDir;
        this.skipPermissions = skipPermissions;
    }

    public void touch() {
        this.lastActivityAt = Instant.now();
    }

    public void attachCliSession(String id) {
        if (id != null && !id.isBlank() && this.cliSessionId == null) {
            this.cliSessionId = id;
        }
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
