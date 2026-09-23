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
import lombok.Getter;
import lombok.Setter;
import org.hibernate.annotations.CreationTimestamp;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/** One task in the run queue, at a dense {@code position}, with what happened when its turn came. */
@Entity
@Table(name = "run_queue_item")
@Getter
public class RunQueueItem {

    public static final int DETAIL_MAX = 500;

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(
            name = "task_id",
            nullable = false,
            updatable = false,
            foreignKey = @ForeignKey(name = "fk_run_queue_item_task"))
    private Task task;

    @Setter
    @Column(name = "position", nullable = false)
    private int position;

    @Enumerated(EnumType.STRING)
    @Column(name = "state", nullable = false, length = 20)
    private RunQueueItemState state = RunQueueItemState.QUEUED;

    /** Why the item is where it is, in a sentence, when that is not obvious from the state. */
    @Column(name = "detail", length = DETAIL_MAX)
    private String detail;

    @Column(name = "started_at")
    private Instant startedAt;

    @Column(name = "finished_at")
    private Instant finishedAt;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    protected RunQueueItem() {
    }

    public RunQueueItem(Task task, int position) {
        this.task = task;
        this.position = position;
    }

    /**
     * Move to {@code next} with an optional reason. Starting stamps {@code startedAt} the first
     * time only, so a task resumed after a hold keeps the moment it first ran; settling stamps
     * {@code finishedAt}; going back to {@code QUEUED} clears it.
     */
    public void moveTo(RunQueueItemState next, String reason) {
        state = next;
        detail = reason == null ? null : truncate(reason.strip());
        Instant now = Instant.now();
        if (next == RunQueueItemState.RUNNING && startedAt == null) {
            startedAt = now;
        }
        finishedAt = next.settled() ? now : null;
    }

    private static String truncate(String text) {
        return text.length() <= DETAIL_MAX ? text : text.substring(0, DETAIL_MAX - 1) + "…";
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof RunQueueItem that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "RunQueueItem[" + position + " " + state + "]";
    }
}
