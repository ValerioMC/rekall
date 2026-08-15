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
import jakarta.persistence.JoinTable;
import jakarta.persistence.ManyToMany;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.OneToOne;
import jakarta.persistence.OrderColumn;
import jakarta.persistence.Table;
import jakarta.persistence.UniqueConstraint;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Pattern;
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
 * <p>{@code label} is what {@code task:report-builder} looks up, {@code title} is what the task
 * is called in a sentence, and {@code description} is what it is about. Renaming the title of a
 * task you have been anchoring for a month leaves the anchor working, which is the whole reason
 * the two are separate columns.
 *
 * <p>The label is unique within its project rather than globally, because two projects routinely
 * have a task with the same one. A bare {@code task:report-builder} that matches in two projects
 * is reported as ambiguous rather than guessed at.
 */
@Entity
@Table(
        name = "task",
        uniqueConstraints = @UniqueConstraint(name = "uq_task_project_label", columnNames = {"project_id", "label"}))
@Getter
public class Task {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @NotBlank
    @Size(max = 160)
    @Pattern(regexp = Slug.PATTERN, message = "must be a slug: lowercase letters, digits, '-', '_' or '.'")
    @Column(name = "label", nullable = false, length = 160)
    @Setter
    private String label;

    @NotBlank
    @Size(max = 200)
    @Column(name = "title", nullable = false, length = 200)
    @Setter
    private String title;

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

    /**
     * The notes that arrive when this task is loaded.
     *
     * <p>The owning side, so attaching and detaching both happen here. No cascade on removal:
     * deleting a task unlinks its notes, and a note still attached elsewhere survives. A note
     * left on no task at all is removed by {@code DocumentService}, which is the only place
     * that can tell the difference.
     */
    @ManyToMany
    @JoinTable(
            name = "document_task",
            joinColumns = @JoinColumn(name = "task_id", foreignKey = @jakarta.persistence.ForeignKey(name = "fk_document_task_task")),
            inverseJoinColumns = @JoinColumn(name = "document_id", foreignKey = @jakarta.persistence.ForeignKey(name = "fk_document_task_document")))
    @OrderColumn(name = "position")
    private List<Document> documents = new ArrayList<>();

    /**
     * What this task's implementation looks like right now, or null while nobody has said.
     *
     * <p>One, at most, and the database holds that shape rather than this field: {@code task_id}
     * on {@code wrapup} is unique. Cascaded and orphan-removed because a wrapup describes this
     * task and has no meaning anywhere else, which is the opposite of a note.
     */
    @OneToOne(mappedBy = "task", cascade = CascadeType.ALL, orphanRemoval = true)
    @Setter
    private Wrapup wrapup;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected Task() {
        // for JPA
    }

    public Task(String label, String title) {
        this.label = label;
        this.title = title;
    }

    /** Both sides are kept in step, so the in-memory graph agrees with what will be flushed. */
    public void attach(Document document) {
        if (documents.contains(document)) {
            return;
        }
        documents.add(document);
        document.getTasks().add(this);
    }

    public void detach(Document document) {
        if (documents.remove(document)) {
            document.getTasks().remove(this);
        }
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
        return "Task[" + label + "]";
    }
}
