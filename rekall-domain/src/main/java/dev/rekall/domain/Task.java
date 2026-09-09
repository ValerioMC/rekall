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
import jakarta.persistence.OneToMany;
import jakarta.persistence.OneToOne;
import jakarta.persistence.OrderBy;
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

    @Enumerated(EnumType.STRING)
    @Column(name = "review_state", nullable = false, length = 20)
    private TaskStepState reviewState = TaskStepState.OPEN;

    @Column(name = "claimed_at")
    private Instant claimedAt;

    @Column(name = "accepted_at")
    private Instant acceptedAt;

    @Size(max = 2_000)
    @Column(name = "review_note", length = 2_000)
    @Setter
    private String reviewNote;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(name = "project_id", nullable = false, foreignKey = @jakarta.persistence.ForeignKey(name = "fk_task_project"))
    @Setter
    private Project project;

    @ManyToMany
    @JoinTable(
            name = "document_task",
            joinColumns = @JoinColumn(name = "task_id", foreignKey = @jakarta.persistence.ForeignKey(name = "fk_document_task_task")),
            inverseJoinColumns = @JoinColumn(name = "document_id", foreignKey = @jakarta.persistence.ForeignKey(name = "fk_document_task_document")))
    @OrderColumn(name = "position")
    private List<Document> documents = new ArrayList<>();

    @OneToMany(mappedBy = "task", cascade = CascadeType.ALL, orphanRemoval = true)
    @OrderBy("position ASC")
    private List<TaskStep> steps = new ArrayList<>();

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
    }

    public Task(String label, String title) {
        this.label = label;
        this.title = title;
    }

    /**
     * True while this task has no checklist. The task-scoped review line
     * ({@link #reviewState} and its two moments) only means anything here: once
     * {@code steps} is not empty the checklist is the source of truth and these
     * columns are kept but ignored.
     */
    public boolean reviewActive() {
        return steps.isEmpty();
    }

    /**
     * Walk the task-scoped review line, the same four values a {@link TaskStep}
     * moves along read at task scope: {@code OPEN} before a session has run or
     * after a send back, {@code RUNNING} while a session is attached to the task
     * anchor with no step target, {@code CLAIMED} once a Claude-authored wrapup
     * lands, {@code DONE} when the console accepts it. Each move rewrites the two
     * moments and the send-back note so they never outlive the state that set
     * them.
     */
    public void markReviewState(TaskStepState next) {
        if (reviewState == next) {
            return;
        }
        reviewState = next;
        Instant now = Instant.now();
        switch (next) {
            case OPEN, RUNNING -> {
                claimedAt = null;
                acceptedAt = null;
                reviewNote = null;
            }
            case CLAIMED -> {
                claimedAt = now;
                acceptedAt = null;
                reviewNote = null;
            }
            case DONE -> {
                if (claimedAt == null) {
                    claimedAt = now;
                }
                acceptedAt = now;
                reviewNote = null;
            }
        }
    }

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
