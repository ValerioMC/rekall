package dev.rekall.meta.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.Table;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;
import lombok.Getter;
import lombok.Setter;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/**
 * A markdown document attached to a record of a dynamically generated entity.
 *
 * <p>{@code (entityName, recordId)} is a soft reference, not a foreign key: it points into
 * {@code rekall_data}, whose tables do not exist when Liquibase runs. The consequence is
 * explicit and handled in the DDL apply step: dropping an entity must also delete its
 * documents, because the database will not do it.
 */
@Entity
@Table(schema = "rekall_meta", name = "document")
@Getter
public class Document {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @NotBlank
    @Size(max = 63)
    @Column(name = "entity_name", nullable = false, length = 63)
    private String entityName;

    @Column(name = "record_id", nullable = false)
    private UUID recordId;

    @NotBlank
    @Size(max = 255)
    @Column(name = "title", nullable = false, length = 255)
    @Setter
    private String title;

    /** Free-form classification: {@code context}, {@code notes}, {@code architecture}, {@code report}. */
    @NotBlank
    @Size(max = 40)
    @Column(name = "kind", nullable = false, length = 40)
    @Setter
    private String kind;

    @Column(name = "body_markdown", nullable = false, columnDefinition = "text")
    @Setter
    private String bodyMarkdown;

    /** Provenance when imported from a folder. Never read back as a source of truth. */
    @Column(name = "source_path", columnDefinition = "text")
    @Setter
    private String sourcePath;

    @Column(name = "position", nullable = false)
    @Setter
    private int position;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected Document() {
        // for JPA
    }

    public Document(String entityName, UUID recordId, String title, String kind, String bodyMarkdown) {
        this.entityName = entityName;
        this.recordId = recordId;
        this.title = title;
        this.kind = kind;
        this.bodyMarkdown = bodyMarkdown;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof Document that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "Document[" + entityName + "/" + recordId + " " + title + "]";
    }
}
