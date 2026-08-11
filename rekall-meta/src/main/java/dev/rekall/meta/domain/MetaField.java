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

import java.util.Objects;
import java.util.UUID;

/** Definition of one column of a dynamically generated entity. */
@Entity
@Table(
        schema = "rekall_meta",
        name = "meta_field",
        uniqueConstraints =
                @UniqueConstraint(
                        name = "uq_meta_field_table_column",
                        columnNames = {"meta_table_id", "column_name"}))
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
    private MetaTable metaTable;

    @NotBlank
    @Size(max = 63)
    @Column(name = "column_name", nullable = false, length = 63, updatable = false)
    private String columnName;

    @NotBlank
    @Size(max = 120)
    @Column(name = "label", nullable = false, length = 120)
    private String label;

    /** Rendered into the schema description Claude reads. Mandatory for the same reason as on the entity. */
    @NotBlank
    @Column(name = "description", nullable = false, columnDefinition = "text")
    private String description;

    @Enumerated(EnumType.STRING)
    @Column(name = "type", nullable = false, length = 20)
    private MetaFieldType type;

    @Column(name = "is_nullable", nullable = false)
    private boolean nullable = true;

    /** Rendered as a bound literal of the mapped type, never as raw SQL. */
    @Column(name = "default_value", columnDefinition = "text")
    private String defaultValue;

    @Column(name = "length")
    private Integer length;

    @Column(name = "precision")
    private Integer precision;

    @Column(name = "scale")
    private Integer scale;

    @Column(name = "enum_values", columnDefinition = "text[]")
    private String[] enumValues;

    @Column(name = "position", nullable = false)
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

    public UUID getId() {
        return id;
    }

    public MetaTable getMetaTable() {
        return metaTable;
    }

    void setMetaTable(MetaTable metaTable) {
        this.metaTable = metaTable;
    }

    public String getColumnName() {
        return columnName;
    }

    public String getLabel() {
        return label;
    }

    public void setLabel(String label) {
        this.label = label;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public MetaFieldType getType() {
        return type;
    }

    public void setType(MetaFieldType type) {
        this.type = type;
    }

    public boolean isNullable() {
        return nullable;
    }

    public void setNullable(boolean nullable) {
        this.nullable = nullable;
    }

    public String getDefaultValue() {
        return defaultValue;
    }

    public void setDefaultValue(String defaultValue) {
        this.defaultValue = defaultValue;
    }

    public Integer getLength() {
        return length;
    }

    public void setLength(Integer length) {
        this.length = length;
    }

    public Integer getPrecision() {
        return precision;
    }

    public void setPrecision(Integer precision) {
        this.precision = precision;
    }

    public Integer getScale() {
        return scale;
    }

    public void setScale(Integer scale) {
        this.scale = scale;
    }

    public String[] getEnumValues() {
        return enumValues == null ? new String[0] : enumValues.clone();
    }

    public void setEnumValues(String[] enumValues) {
        this.enumValues = enumValues == null ? null : enumValues.clone();
    }

    public int getPosition() {
        return position;
    }

    void setPosition(int position) {
        this.position = position;
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
