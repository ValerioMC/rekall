package dev.rekall.meta.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.Table;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/**
 * One executed DDL statement.
 *
 * <p>{@code rekall_data} has no Liquibase changelog by construction, so this table is what
 * replaces it: it is both the audit trail and the recipe to rebuild the generated schema
 * from an empty database.
 */
@Entity
@Table(schema = "rekall_meta", name = "ddl_log")
public class DdlLog {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    /** Groups every statement produced by a single Execute. */
    @Column(name = "plan_id", nullable = false)
    private UUID planId;

    /** Position within the plan. Ordering is what makes a replay reproducible. */
    @Column(name = "sequence", nullable = false)
    private int sequence;

    @Column(name = "statement", nullable = false, columnDefinition = "text")
    private String statement;

    @Enumerated(EnumType.STRING)
    @Column(name = "status", nullable = false, length = 20)
    private DdlStatus status;

    @Column(name = "error", columnDefinition = "text")
    private String error;

    @Column(name = "applied_at", nullable = false)
    private Instant appliedAt;

    protected DdlLog() {
        // for JPA
    }

    public DdlLog(UUID planId, int sequence, String statement, DdlStatus status, String error) {
        this.planId = planId;
        this.sequence = sequence;
        this.statement = statement;
        this.status = status;
        this.error = error;
        this.appliedAt = Instant.now();
    }

    public UUID getId() {
        return id;
    }

    public UUID getPlanId() {
        return planId;
    }

    public int getSequence() {
        return sequence;
    }

    public String getStatement() {
        return statement;
    }

    public DdlStatus getStatus() {
        return status;
    }

    public String getError() {
        return error;
    }

    public Instant getAppliedAt() {
        return appliedAt;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof DdlLog that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }
}
