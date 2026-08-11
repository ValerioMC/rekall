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
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;
import lombok.Getter;
import lombok.Setter;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/**
 * A markdown note attached to exactly one project, task or environment.
 *
 * <p>Three nullable foreign keys with a check constraint, rather than the
 * {@code (entityName, recordId)} pair the meta-model needed. The owners are known at compile
 * time now, so the database can enforce that a note never outlives what it documents, and a
 * project reaches its notes by navigating an association instead of by running a query.
 */
@Entity
@Table(name = "document")
@Getter
public class Document {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

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

    @Column(name = "body_markdown", nullable = false, length = 100_000)
    @Setter
    private String bodyMarkdown;

    /** Provenance when imported from a folder. Never read back as a source of truth. */
    @Column(name = "source_path", length = 500)
    @Setter
    private String sourcePath;

    @Column(name = "position", nullable = false)
    @Setter
    private int position;

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "project_id", foreignKey = @ForeignKey(name = "fk_document_project"))
    private Project project;

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "task_id", foreignKey = @ForeignKey(name = "fk_document_task"))
    private Task task;

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "environment_id", foreignKey = @ForeignKey(name = "fk_document_environment"))
    private Environment environment;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected Document() {
        // for JPA
    }

    public Document(String title, String kind, String bodyMarkdown) {
        this.title = title;
        this.kind = kind;
        this.bodyMarkdown = bodyMarkdown;
    }

    void attachTo(Project owner) {
        clearOwners();
        this.project = owner;
    }

    void attachTo(Task owner) {
        clearOwners();
        this.task = owner;
    }

    void attachTo(Environment owner) {
        clearOwners();
        this.environment = owner;
    }

    /** Exactly one owner is set, which the check constraint also enforces in the database. */
    private void clearOwners() {
        this.project = null;
        this.task = null;
        this.environment = null;
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
        return "Document[" + title + "]";
    }
}
