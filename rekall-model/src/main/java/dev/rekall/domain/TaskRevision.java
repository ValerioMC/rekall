package dev.rekall.domain;

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

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/**
 * An earlier version of a task's wrapup or description, kept when something replaced or deleted
 * it. Immutable once written: a revision is a copy, never edited, and restoring one writes it
 * back as the current text rather than changing the revision.
 */
@Entity
@Table(name = "task_revision")
@Getter
public class TaskRevision {

    public static final int MAX_CHARACTERS = 100_000;

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(
            name = "task_id",
            nullable = false,
            updatable = false,
            foreignKey = @ForeignKey(name = "fk_task_revision_task"))
    private Task task;

    @Enumerated(EnumType.STRING)
    @Column(name = "kind", nullable = false, updatable = false, length = 20)
    private RevisionKind kind;

    @Column(name = "body_markdown", nullable = false, updatable = false, length = MAX_CHARACTERS)
    private String bodyMarkdown;

    /** Who wrote the version kept here; known for a wrapup, absent for a description. */
    @Enumerated(EnumType.STRING)
    @Column(name = "written_by", updatable = false, length = 20)
    private WrapupAuthor writtenBy;

    /** When the version kept here was written, when that is known. */
    @Column(name = "written_at", updatable = false)
    private Instant writtenAt;

    /** When it stopped being the current text. */
    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    protected TaskRevision() {
    }

    public TaskRevision(Task task, RevisionKind kind, String bodyMarkdown, WrapupAuthor writtenBy, Instant writtenAt) {
        this.task = task;
        this.kind = kind;
        this.bodyMarkdown = bodyMarkdown;
        this.writtenBy = writtenBy;
        this.writtenAt = writtenAt;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof TaskRevision that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "TaskRevision[" + kind + " of " + (task == null ? "?" : task.getLabel()) + "]";
    }
}
