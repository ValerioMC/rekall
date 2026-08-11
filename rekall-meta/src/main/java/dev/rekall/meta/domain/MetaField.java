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
import jakarta.persistence.UniqueConstraint;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;
import lombok.AccessLevel;
import lombok.Getter;
import lombok.Setter;

import java.util.Objects;
import java.util.UUID;

/**
 * Definition of one column of a dynamically generated entity.
 *
 * <p>{@code @Setter} is applied field by field. {@code columnName} has none, for the same
 * reason {@code physicalName} has none on {@link MetaTable}: the identifier is immutable and
 * a class-level {@code @Setter} would generate the method that breaks it. {@code metaTable}
 * and {@code position} keep package visibility so that only {@link MetaTable#addField} can
 * maintain the two sides of the association.
 */
@Entity
@Table(
        schema = "rekall_meta",
        name = "meta_field",
        uniqueConstraints =
                @UniqueConstraint(
                        name = "uq_meta_field_table_column",
                        columnNames = {"meta_table_id", "column_name"}))
@Getter
public class MetaField {

    /** Default {@code varchar} length when the user does not pick one. */
    public static final int DEFAULT_TEXT_LENGTH = 255;

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    /**
     * Eager on purpose. The schema registry hands these instances out beyond the persistence
     * context, and a lazy proxy here would turn any traversal back to the owning entity into a
     * {@code LazyInitializationException} at an unpredictable call site. The meta-model is a
     * handful of rows, so there is nothing to gain by deferring it.
     */
    @ManyToOne(fetch = FetchType.EAGER, optional = false)
    @JoinColumn(name = "meta_table_id", nullable = false)
    @Setter(AccessLevel.PACKAGE)
    private MetaTable metaTable;

    @NotBlank
    @Size(max = 63)
    @Column(name = "column_name", nullable = false, length = 63, updatable = false)
    private String columnName;

    @NotBlank
    @Size(max = 120)
    @Column(name = "label", nullable = false, length = 120)
    @Setter
    private String label;

    /** Rendered into the schema description Claude reads. Mandatory for the same reason as on the entity. */
    @NotBlank
    @Column(name = "description", nullable = false, columnDefinition = "text")
    @Setter
    private String description;

    @Enumerated(EnumType.STRING)
    @Column(name = "type", nullable = false, length = 20)
    @Setter
    private MetaFieldType type;

    @Column(name = "is_nullable", nullable = false)
    @Setter
    private boolean nullable = true;

    /** Rendered as a bound literal of the mapped type, never as raw SQL. */
    @Column(name = "default_value", columnDefinition = "text")
    @Setter
    private String defaultValue;

    @Column(name = "length")
    @Setter
    private Integer length;

    @Column(name = "precision")
    @Setter
    private Integer precision;

    @Column(name = "scale")
    @Setter
    private Integer scale;

    @Column(name = "enum_values", columnDefinition = "text[]")
    private String[] enumValues;

    @Column(name = "position", nullable = false)
    @Setter(AccessLevel.PACKAGE)
    private int position;

    protected MetaField() {
        // for JPA
    }

    public MetaField(String columnName, String label, String description, MetaFieldType type) {
        this.columnName = columnName;
        this.label = label;
        this.description = description;
        this.type = type;
    }

    /** Effective {@code varchar} length, falling back to {@link #DEFAULT_TEXT_LENGTH}. */
    public int effectiveLength() {
        return length == null ? DEFAULT_TEXT_LENGTH : length;
    }

    public int effectivePrecision() {
        return precision == null ? 19 : precision;
    }

    public int effectiveScale() {
        return scale == null ? 4 : scale;
    }

    /** Defensive copy in both directions. Lombok would hand out the field itself. */
    public String[] getEnumValues() {
        return enumValues == null ? new String[0] : enumValues.clone();
    }

    public void setEnumValues(String[] enumValues) {
        this.enumValues = enumValues == null ? null : enumValues.clone();
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof MetaField that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "MetaField[" + columnName + " " + type + "]";
    }
}
