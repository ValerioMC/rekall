package dev.rekall.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.Table;
import lombok.Getter;
import lombok.Setter;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/**
 * The one run queue: when it starts, the usage ceiling it stops at, and how the sessions it opens
 * are launched. There is a single row; the queued tasks are {@link RunQueueItem}s.
 */
@Entity
@Table(name = "run_queue")
@Getter
@Setter
public class RunQueue {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @Enumerated(EnumType.STRING)
    @Column(name = "state", nullable = false, length = 20)
    private RunQueueState state = RunQueueState.IDLE;

    /** When a scheduled queue starts; null for one started at once or not started at all. */
    @Column(name = "start_at")
    private Instant startAt;

    /** The usage percentage at which no new task or step is started; null means no ceiling. */
    @Column(name = "ceiling_percent")
    private Integer ceilingPercent;

    @Column(name = "skip_permissions", nullable = false)
    private boolean skipPermissions;

    @Column(name = "model", length = 20)
    private String model;

    @Column(name = "effort", length = 20)
    private String effort;

    /** While {@code HOLDING}: when the queue looks at usage again. */
    @Column(name = "hold_until")
    private Instant holdUntil;

    @Column(name = "hold_reason", length = 500)
    private String holdReason;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    /** Move to {@code next}, clearing whatever belonged only to the state being left. */
    public void moveTo(RunQueueState next) {
        state = next;
        if (next != RunQueueState.SCHEDULED) {
            startAt = null;
        }
        if (next != RunQueueState.HOLDING) {
            holdUntil = null;
            holdReason = null;
        }
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof RunQueue that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "RunQueue[" + state + "]";
    }
}
