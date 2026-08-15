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
import jakarta.persistence.OneToOne;
import jakarta.persistence.Table;
import jakarta.persistence.UniqueConstraint;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;
import lombok.Getter;
import lombok.Setter;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/**
 * What a task's implementation looks like right now.
 *
 * <p>Not a log and not a diff. A wrapup answers "what does this do, as it stands", so the next
 * session opens on the current shape of the work rather than on a trail of what changed when.
 * A note describes something you learned; a wrapup describes something you built, and it is
 * replaced wholesale every time rather than appended to. That is the whole of the rule, and it
 * is why {@code body_markdown} has no history table behind it.
 *
 * <p>Exactly one per task, and the database says so: {@code task_id} is {@code NOT NULL} and
 * unique. A second wrapup on a task would be two answers to a question that has one, and a
 * service check would let a second writer slip one in between the read and the insert.
 *
 * <p>{@code ON DELETE CASCADE} on the task, because a wrapup describes a task and nothing else.
 */
@Entity
@Table(name = "wrapup", uniqueConstraints = @UniqueConstraint(name = "uq_wrapup_task", columnNames = "task_id"))
@Getter
public class Wrapup {

    /**
     * The cap, in characters.
     *
     * <p>Deliberately far below the 100,000 a note may hold. A wrapup that no longer fits on a
     * screen has stopped being a description of the state and become a record of the process,
     * which is the one thing it must not be. The limit is where that turn is caught.
     */
    public static final int MAX_CHARACTERS = 20_000;

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @OneToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(
            name = "task_id",
            nullable = false,
            unique = true,
            updatable = false,
            foreignKey = @ForeignKey(name = "fk_wrapup_task"))
    private Task task;

    @NotBlank
    @Size(max = MAX_CHARACTERS)
    @Column(name = "body_markdown", nullable = false, length = MAX_CHARACTERS)
    @Setter
    private String bodyMarkdown;

    @Enumerated(EnumType.STRING)
    @Column(name = "written_by", nullable = false, length = 20)
    @Setter
    private WrapupAuthor writtenBy;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected Wrapup() {
        // for JPA
    }

    public Wrapup(Task task, String bodyMarkdown, WrapupAuthor writtenBy) {
        this.task = task;
        this.bodyMarkdown = bodyMarkdown;
        this.writtenBy = writtenBy;
    }

    /** What you would type after {@code /rk} to load the task this describes. */
    public String anchor() {
        return "project:%s task:%s".formatted(task.getProject().getLabel(), task.getLabel());
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof Wrapup that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "Wrapup[" + (task == null ? "?" : task.getLabel()) + "]";
    }
}
