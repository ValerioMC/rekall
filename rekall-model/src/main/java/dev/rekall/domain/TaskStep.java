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
@Table(name = "task_step")
@Getter
public class TaskStep {

    public static final int MAX_CHARACTERS = 20_000;

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(
            name = "task_id",
            nullable = false,
            updatable = false,
            foreignKey = @ForeignKey(name = "fk_task_step_task"))
    private Task task;

    @NotBlank
    @Size(max = 200)
    @Column(name = "title", nullable = false, length = 200)
    @Setter
    private String title;

    @Size(max = MAX_CHARACTERS)
    @Column(name = "body_markdown", length = MAX_CHARACTERS)
    @Setter
    private String bodyMarkdown;

    @Enumerated(EnumType.STRING)
    @Column(name = "state", nullable = false, length = 20)
    private TaskStepState state = TaskStepState.DRAFT;

    @Column(name = "running_at")
    private Instant runningAt;

    @Column(name = "claimed_at")
    private Instant claimedAt;

    @Column(name = "done_at")
    private Instant doneAt;

    @Column(name = "position", nullable = false)
    @Setter
    private int position;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected TaskStep() {
    }

    public TaskStep(Task task, String title, int position) {
        this.task = task;
        this.title = title;
        this.position = position;
    }

    public boolean isDone() {
        return state == TaskStepState.DONE;
    }

    public Instant completedAt() {
        return claimedAt != null ? claimedAt : doneAt;
    }

    public void markState(TaskStepState next) {
        if (state == next) {
            return;
        }
        state = next;
        Instant now = Instant.now();
        switch (next) {
            case DRAFT, OPEN -> {
                runningAt = null;
                claimedAt = null;
                doneAt = null;
            }
            case RUNNING -> {
                runningAt = now;
                claimedAt = null;
                doneAt = null;
            }
            case CLAIMED -> {
                if (runningAt == null) {
                    runningAt = now;
                }
                claimedAt = now;
                doneAt = null;
            }
            case DONE -> doneAt = now;
        }
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof TaskStep that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "TaskStep[" + title + ", " + state + "]";
    }
}
