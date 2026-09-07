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

/**
 * One piece of a task, somewhere on the line from open to done.
 *
 * <p>This is the answer to the question neither the description nor the wrapup can give. The
 * description is the brief and grows as the work is redefined; the wrapup is the state of the
 * implementation as prose. Between the two, "what is left" had to be worked out by reading both
 * and comparing them, which is slow for a person and unreliable for a model. A step says it
 * outright.
 *
 * <p>{@code bodyMarkdown} is where the implementation detail of that one piece goes, and it is
 * the reason a step is a row rather than a line in the description: an open step is handed to
 * Claude in full and a closed one as its title alone, so the context carries the detail of the
 * work that remains and only the names of the work that is finished.
 *
 * <p>{@link #state} carries where the work has got to. A session moves a step to
 * {@link TaskStepState#RUNNING} when it starts and {@link TaskStepState#CLAIMED} when it
 * finishes, over MCP, and that is as far as it can push: the move to {@link TaskStepState#DONE}
 * is a person in the console saying they reviewed the work. A model that could tick its own box
 * would be answering the question it was asked.
 */
@Entity
@Table(name = "task_step")
@Getter
public class TaskStep {

    /**
     * The cap on the detail, in characters.
     *
     * <p>The wrapup's number, for the same reason: a step whose detail no longer fits on a
     * screen is not a step, it is a task, and the model already has a level for that.
     */
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

    /**
     * Where the work has got to. Never null: a step is created {@link TaskStepState#OPEN} and
     * the transitions below keep it one of the four.
     */
    @Enumerated(EnumType.STRING)
    @Column(name = "state", nullable = false, length = 20)
    private TaskStepState state = TaskStepState.OPEN;

    /** When a session picked this step up, or null if it never has since it was last open. */
    @Column(name = "running_at")
    private Instant runningAt;

    /** When a session claimed this step as finished, or null if it has not. */
    @Column(name = "claimed_at")
    private Instant claimedAt;

    /** When a person accepted the work, or null while it is anything short of done. */
    @Column(name = "done_at")
    private Instant doneAt;

    /**
     * Where it sits in the list, from zero.
     *
     * <p>Maintained by {@code TaskStepService} rather than by an {@code @OrderColumn}: the
     * service is the only writer, renumbering is one loop over a list it has already loaded,
     * and a column Hibernate maintains from the inverse side of the association is the kind of
     * thing that works until two of them disagree.
     */
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
        // for JPA
    }

    public TaskStep(Task task, String title, int position) {
        this.task = task;
        this.title = title;
        this.position = position;
    }

    /** Whether a person has accepted the work. The navigator's progress count is built on this. */
    public boolean isDone() {
        return state == TaskStepState.DONE;
    }

    /**
     * The moment this step's work was finished, whoever still has to sign off.
     *
     * <p>{@code claimedAt} when a session claimed it, {@code doneAt} otherwise. It is what a
     * wrapup written mid-run is measured against: a wrapup written right after a step is claimed
     * already accounts for it, and a later console tick moving it to done does not change what
     * the implementation is.
     */
    public Instant completedAt() {
        return claimedAt != null ? claimedAt : doneAt;
    }

    /**
     * Moves the step to a state, and keeps the three timestamps saying what they say.
     *
     * <p>One method rather than a setter per field, because the state and its moment are one
     * fact. Moving back to {@link TaskStepState#OPEN} clears all three: a step that was reopened
     * has no history of being anything else, the same way the boolean this replaced cleared its
     * {@code doneAt}.
     */
    public void markState(TaskStepState next) {
        if (state == next) {
            return;
        }
        state = next;
        Instant now = Instant.now();
        switch (next) {
            case OPEN -> {
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
