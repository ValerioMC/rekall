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
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;

/** Definition of one dynamically generated entity. */
@Entity
@Table(
        schema = "rekall_meta",
        name = "meta_table",
        uniqueConstraints = @UniqueConstraint(name = "uq_meta_table_physical_name", columnNames = "physical_name"))
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
    private String label;

    @NotBlank
    @Size(max = 120)
    @Column(name = "label_plural", nullable = false, length = 120)
    private String labelPlural;

    /**
     * Rendered into the schema description Claude reads before choosing an entity. A vague
     * description degrades answer quality directly, which is why it is mandatory rather than
     * optional documentation.
     */
    @NotBlank
    @Column(name = "description", nullable = false, columnDefinition = "text")
    private String description;

    /** Alternative names the user might use in a question, e.g. {@code progetti, commesse}. */
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
    private UUID displayFieldId;

    @Enumerated(EnumType.STRING)
    @Column(name = "status", nullable = false, length = 20)
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

    public UUID getId() {
        return id;
    }

    public String getPhysicalName() {
        return physicalName;
    }

    public String getLabel() {
        return label;
    }

    public void setLabel(String label) {
        this.label = label;
    }

    public String getLabelPlural() {
        return labelPlural;
    }

    public void setLabelPlural(String labelPlural) {
        this.labelPlural = labelPlural;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public String[] getAliases() {
        return aliases == null ? new String[0] : aliases.clone();
    }

    public void setAliases(String[] aliases) {
        this.aliases = aliases == null ? new String[0] : aliases.clone();
    }

    public UUID getDisplayFieldId() {
        return displayFieldId;
    }

    public void setDisplayFieldId(UUID displayFieldId) {
        this.displayFieldId = displayFieldId;
    }

    public MetaTableStatus getStatus() {
        return status;
    }

    public void setStatus(MetaTableStatus status) {
        this.status = status;
    }

    public List<MetaField> getFields() {
        return fields;
    }

    public Instant getCreatedAt() {
        return createdAt;
    }

    public Instant getUpdatedAt() {
        return updatedAt;
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
