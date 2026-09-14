package dev.rekall.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.FetchType;
import jakarta.persistence.ForeignKey;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.Lob;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.Table;
import lombok.Getter;
import org.hibernate.annotations.CreationTimestamp;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

@Entity
@Table(name = "commit_reference")
@Getter
public class CommitReference {

    public static final int HASH_MAX = 40;
    public static final int COMMENT_MAX = 200;
    public static final int DIFF_MAX = 200_000;

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(
            name = "task_id",
            nullable = false,
            updatable = false,
            foreignKey = @ForeignKey(name = "fk_commit_reference_task"))
    private Task task;

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(
            name = "step_id",
            updatable = false,
            foreignKey = @ForeignKey(name = "fk_commit_reference_step"))
    private TaskStep step;

    @Column(name = "commit_hash", nullable = false, updatable = false, length = HASH_MAX)
    private String commitHash;

    @Column(name = "comment", nullable = false, updatable = false, length = COMMENT_MAX)
    private String comment;

    /** The diff this commit introduced, so the change can be reread later without a checkout. Null for a commit logged before this column existed, or when git could not produce one. */
    @Lob
    @Column(name = "diff", updatable = false)
    private String diff;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    protected CommitReference() {
    }

    public CommitReference(Task task, TaskStep step, String commitHash, String comment, String diff) {
        this.task = task;
        this.step = step;
        this.commitHash = commitHash;
        this.comment = comment;
        this.diff = diff;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof CommitReference that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "CommitReference[" + commitHash + " " + task.getLabel() + "]";
    }
}
