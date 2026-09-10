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

    @Column(name = "auto_wrapup", nullable = false)
    private boolean autoWrapup = false;

    @Size(max = 2_000)
    @Column(name = "wrapup_directive", length = 2_000)
    private String wrapupDirective;

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
     * True while the task has no checklist (only draft steps count as none). Once a step is past
     * {@code DRAFT} the checklist is the source of truth and the review columns are ignored.
     */
    public boolean reviewActive() {
        return steps.stream().allMatch(step -> step.getState() == TaskStepState.DRAFT);
    }

    /**
     * Move the task-scoped review line: {@code OPEN} before a run or after a send back,
     * {@code RUNNING} during a session, {@code CLAIMED} once a wrapup lands, {@code DONE} on accept.
     * Each move resets {@link #claimedAt}, {@link #acceptedAt} and {@link #reviewNote} to match.
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

    /**
     * The standing wrapup instruction. While {@code auto} is on, {@code /rk ... wrapup} runs
     * unprompted and follows {@code directive}. A blank directive is stored as none, and the
     * directive is cleared when {@code auto} goes off.
     */
    public void configureWrapup(boolean auto, String directive) {
        this.autoWrapup = auto;
        String trimmed = directive == null || directive.isBlank() ? null : directive.trim();
        this.wrapupDirective = auto ? trimmed : null;
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
