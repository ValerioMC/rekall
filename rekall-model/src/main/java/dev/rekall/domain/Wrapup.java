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

@Entity
@Table(name = "wrapup", uniqueConstraints = @UniqueConstraint(name = "uq_wrapup_task", columnNames = "task_id"))
@Getter
public class Wrapup {

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
    }

    public Wrapup(Task task, String bodyMarkdown, WrapupAuthor writtenBy) {
        this.task = task;
        this.bodyMarkdown = bodyMarkdown;
        this.writtenBy = writtenBy;
    }

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
