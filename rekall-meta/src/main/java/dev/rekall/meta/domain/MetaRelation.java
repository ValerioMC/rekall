package dev.rekall.meta.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.FetchType;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.Table;
import jakarta.validation.constraints.NotBlank;
import org.hibernate.annotations.CreationTimestamp;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/** Definition of a foreign key relation between two dynamically generated entities. */
@Entity
@Table(schema = "rekall_meta", name = "meta_relation")
public class MetaRelation {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    /** Eager for the same reason as {@code MetaField.metaTable}: these leave the session. */
    @ManyToOne(fetch = FetchType.EAGER, optional = false)
    @JoinColumn(name = "source_table_id", nullable = false)
    private MetaTable sourceTable;

    @ManyToOne(fetch = FetchType.EAGER, optional = false)
    @JoinColumn(name = "target_table_id", nullable = false)
    private MetaTable targetTable;

    @Enumerated(EnumType.STRING)
    @Column(name = "kind", nullable = false, length = 20)
    private RelationKind kind;

    /** The {@code REFERENCE} field carrying the foreign key. Set for {@code MANY_TO_ONE} only. */
    @Column(name = "source_field_id")
    private UUID sourceFieldId;

    /** Physical name of the generated join table. Set for {@code MANY_TO_MANY} only. */
    @Column(name = "join_table_name", length = 63)
    private String joinTableName;

    @Enumerated(EnumType.STRING)
    @Column(name = "on_delete", nullable = false, length = 20)
    private OnDeleteAction onDelete = OnDeleteAction.RESTRICT;

    /** Rendered into the schema description, e.g. "The environment this task runs on". */
    @NotBlank
    @Column(name = "description", nullable = false, columnDefinition = "text")
    private String description;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    protected MetaRelation() {
        // for JPA
    }

    public static MetaRelation manyToOne(
            MetaTable source, MetaTable target, UUID sourceFieldId, OnDeleteAction onDelete, String description) {
        MetaRelation relation = new MetaRelation();
        relation.sourceTable = source;
        relation.targetTable = target;
        relation.kind = RelationKind.MANY_TO_ONE;
        relation.sourceFieldId = sourceFieldId;
        relation.onDelete = onDelete;
        relation.description = description;
        return relation;
    }

    public static MetaRelation manyToMany(
            MetaTable source, MetaTable target, String joinTableName, String description) {
        MetaRelation relation = new MetaRelation();
        relation.sourceTable = source;
        relation.targetTable = target;
        relation.kind = RelationKind.MANY_TO_MANY;
        relation.joinTableName = joinTableName;
        // A join table row is meaningless once either side is gone.
        relation.onDelete = OnDeleteAction.CASCADE;
        relation.description = description;
        return relation;
    }

    /** Physical name of the foreign key constraint, stable across replays of the plan. */
    public String constraintName() {
        return "fk_" + sourceTable.getPhysicalName() + "_" + targetTable.getPhysicalName() + "_" + shortId();
    }

    private String shortId() {
        return id == null ? "new" : id.toString().substring(0, 8);
    }

    public UUID getId() {
        return id;
    }

    public MetaTable getSourceTable() {
        return sourceTable;
    }

    public MetaTable getTargetTable() {
        return targetTable;
    }

    public RelationKind getKind() {
        return kind;
    }

    public UUID getSourceFieldId() {
        return sourceFieldId;
    }

    public String getJoinTableName() {
        return joinTableName;
    }

    public OnDeleteAction getOnDelete() {
        return onDelete;
    }

    public void setOnDelete(OnDeleteAction onDelete) {
        this.onDelete = onDelete;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public Instant getCreatedAt() {
        return createdAt;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof MetaRelation that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "MetaRelation[" + sourceTable.getPhysicalName() + " -> " + targetTable.getPhysicalName() + "]";
    }
}
