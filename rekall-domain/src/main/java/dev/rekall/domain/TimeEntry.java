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
import lombok.Getter;
import lombok.Setter;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/**
 * One sitting of work on a task, from the moment it was picked up to the moment it was put
 * down.
 *
 * <p>A task worked across several days is several rows, not one running total: a total can only
 * be corrected by editing it, and a session with a start and a stop can be corrected by editing
 * either. {@code stoppedAt} is null for exactly as long as the session is open, and only one row
 * across the whole application may be in that state at a time — {@code TimeEntryService} is
 * what enforces it, the same way a second wrapup on a task is refused in code rather than by a
 * constraint.
 *
 * <p>{@code ON DELETE CASCADE} on the task, because a session describes a task and nothing else.
 */
@Entity
@Table(name = "time_entry")
@Getter
public class TimeEntry {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(
            name = "task_id",
            nullable = false,
            updatable = false,
            foreignKey = @ForeignKey(name = "fk_time_entry_task"))
    private Task task;

    @Column(name = "started_at", nullable = false)
    @Setter
    private Instant startedAt;

    /** Null while the session is still open. */
    @Column(name = "stopped_at")
    @Setter
    private Instant stoppedAt;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected TimeEntry() {
        // for JPA
    }

    public TimeEntry(Task task, Instant startedAt) {
        this.task = task;
        this.startedAt = startedAt;
    }

    /** What you would type after {@code /rk} to load the task this session was spent on. */
    public String anchor() {
        return "project:%s task:%s".formatted(task.getProject().getLabel(), task.getLabel());
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof TimeEntry that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "TimeEntry[" + (task == null ? "?" : task.getLabel()) + " " + startedAt + "]";
    }
}
