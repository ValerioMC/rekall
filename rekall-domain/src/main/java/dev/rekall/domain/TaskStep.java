package dev.rekall.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
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
 * One piece of a task that is either done or not done yet.
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
 * <p>Only the console writes here. There is no MCP path in, deliberately: a model that could
 * tick its own boxes would be answering the question it was asked. The person who reviewed the
 * work says what is done.
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

    @Column(name = "done", nullable = false)
    private boolean done;

    /** When it was ticked, or null while it is open. Cleared again if it is reopened. */
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

    /**
     * Ticking and unticking, with the timestamp that goes with it.
     *
     * <p>One method rather than two setters, because the flag and the moment are one fact:
     * setting the first without the second leaves a step that is done at no time, and nothing
     * downstream can tell that from a step that was done before the column existed.
     */
    public void markDone(boolean value) {
        if (done == value) {
            return;
        }
        done = value;
        doneAt = value ? Instant.now() : null;
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
        return "TaskStep[" + title + (done ? ", done]" : "]");
    }
}
