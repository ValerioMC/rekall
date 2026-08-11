package dev.rekall.domain;

import jakarta.persistence.CascadeType;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.FetchType;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.OneToMany;
import jakarta.persistence.OrderBy;
import jakarta.persistence.Table;
import jakarta.persistence.UniqueConstraint;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;
import lombok.Getter;
import lombok.Setter;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.UUID;

/**
 * One piece of work on a project, and the unit a session is opened around.
 *
 * <p>{@code name} is unique within its project rather than globally, because two projects
 * routinely have a task with the same name. A bare {@code task:code-validator} that matches in
 * two projects is reported as ambiguous rather than guessed at.
 */
@Entity
@Table(
        name = "task",
        uniqueConstraints = @UniqueConstraint(name = "uq_task_project_name", columnNames = {"project_id", "name"}))
@Getter
public class Task {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @NotBlank
    @Size(max = 160)
    @Column(name = "name", nullable = false, length = 160)
    @Setter
    private String name;

    @Enumerated(EnumType.STRING)
    @Column(name = "status", nullable = false, length = 20)
    @Setter
    private TaskStatus status = TaskStatus.TODO;

    @Column(name = "description", length = 100_000)
    @Setter
    private String description;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(name = "project_id", nullable = false, foreignKey = @jakarta.persistence.ForeignKey(name = "fk_task_project"))
    @Setter
    private Project project;

    /** The cluster and namespace this task runs against. Optional: not every task has one. */
    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "environment_id", foreignKey = @jakarta.persistence.ForeignKey(name = "fk_task_environment"))
    @Setter
    private Environment environment;

    @OneToMany(mappedBy = "task", cascade = CascadeType.ALL, orphanRemoval = true)
    @OrderBy("position ASC")
    private List<Document> documents = new ArrayList<>();

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected Task() {
        // for JPA
    }

    public Task(String name) {
        this.name = name;
    }

    public void addDocument(Document document) {
        document.attachTo(this);
        documents.add(document);
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof Task that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "Task[" + name + "]";
    }
}
