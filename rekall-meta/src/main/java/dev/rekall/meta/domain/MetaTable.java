package dev.rekall.meta.domain;

import jakarta.persistence.CascadeType;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.OneToMany;
import jakarta.persistence.OrderBy;
import jakarta.persistence.Table;
import jakarta.persistence.UniqueConstraint;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;
import lombok.Getter;
import lombok.Setter;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;

/**
 * Definition of one dynamically generated entity.
 *
 * <p>{@code @Setter} is applied field by field and never to the class. {@code physicalName}
 * has no setter on purpose: a diff between the meta-model and {@code information_schema}
 * cannot tell a rename from a drop plus an add, so the identifier is immutable and only the
 * label is editable. A class-level {@code @Setter} would generate the one method that breaks
 * that guarantee.
 *
 * <p>{@code equals}, {@code hashCode} and {@code toString} stay hand-written on the id alone.
 * Generated ones would walk {@code fields} and force the collection to load.
 */
@Entity
@Table(
        schema = "rekall_meta",
        name = "meta_table",
        uniqueConstraints = @UniqueConstraint(name = "uq_meta_table_physical_name", columnNames = "physical_name"))
@Getter
public class MetaTable {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @NotBlank
    @Size(max = 63)
    @Column(name = "physical_name", nullable = false, length = 63, updatable = false)
    private String physicalName;

    @NotBlank
    @Size(max = 120)
    @Column(name = "label", nullable = false, length = 120)
    @Setter
    private String label;

    @NotBlank
    @Size(max = 120)
    @Column(name = "label_plural", nullable = false, length = 120)
    @Setter
    private String labelPlural;

    /**
     * Rendered into the schema description Claude reads before choosing an entity. A vague
     * description degrades answer quality directly, which is why it is mandatory rather than
     * optional documentation.
     */
    @NotBlank
    @Column(name = "description", nullable = false, columnDefinition = "text")
    @Setter
    private String description;

    /**
     * Alternative names the user might use in a question, e.g. {@code progetti, commesse}.
     *
     * <p>No Lombok accessor: the pair below copies the array in both directions. A generated
     * one would hand out the field itself, and a caller mutating it would change the entity.
     */
    @Column(name = "aliases", columnDefinition = "text[]")
    private String[] aliases = new String[0];

    /**
     * Points at the {@link MetaField} that gives a record its human-readable identity.
     *
     * <p>Held as a bare id rather than an association: {@code meta_field} points back at
     * {@code meta_table}, and modelling both directions as JPA associations would force
     * Hibernate to resolve a circular insert order for no benefit.
     */
    @Column(name = "display_field_id")
    @Setter
    private UUID displayFieldId;

    @Enumerated(EnumType.STRING)
    @Column(name = "status", nullable = false, length = 20)
    @Setter
    private MetaTableStatus status = MetaTableStatus.DRAFT;

    @OneToMany(mappedBy = "metaTable", cascade = CascadeType.ALL, orphanRemoval = true)
    @OrderBy("position ASC")
    private List<MetaField> fields = new ArrayList<>();

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected MetaTable() {
        // for JPA
    }

    public MetaTable(String physicalName, String label, String labelPlural, String description) {
        this.physicalName = physicalName;
        this.label = label;
        this.labelPlural = labelPlural;
        this.description = description;
    }

    public void addField(MetaField field) {
        field.setMetaTable(this);
        field.setPosition(fields.size());
        fields.add(field);
    }

    public void removeField(MetaField field) {
        fields.remove(field);
        field.setMetaTable(null);
        reindexPositions();
    }

    private void reindexPositions() {
        for (int i = 0; i < fields.size(); i++) {
            fields.get(i).setPosition(i);
        }
    }

    public Optional<MetaField> findField(String columnName) {
        return fields.stream().filter(f -> f.getColumnName().equals(columnName)).findFirst();
    }

    public Optional<MetaField> displayField() {
        if (displayFieldId == null) {
            return Optional.empty();
        }
        return fields.stream().filter(f -> displayFieldId.equals(f.getId())).findFirst();
    }

    /** Defensive copy in both directions. Lombok would hand out the field itself. */
    public String[] getAliases() {
        return aliases == null ? new String[0] : aliases.clone();
    }

    public void setAliases(String[] aliases) {
        this.aliases = aliases == null ? new String[0] : aliases.clone();
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof MetaTable that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "MetaTable[" + physicalName + "]";
    }
}
