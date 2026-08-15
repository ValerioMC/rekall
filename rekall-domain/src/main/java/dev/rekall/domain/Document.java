package dev.rekall.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.ManyToMany;
import jakarta.persistence.Table;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;
import lombok.Getter;
import lombok.Setter;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.Instant;
import java.util.LinkedHashSet;
import java.util.Objects;
import java.util.Set;
import java.util.UUID;

/**
 * A markdown note, attached to any number of tasks.
 *
 * <p>It used to belong to exactly one owner, which meant a note that mattered to three tasks
 * had to be written three times and the three copies drifted. Cluster access, a naming
 * convention and an onboarding step are all notes of that kind, so the relation is the shape
 * the content already had.
 *
 * <p>{@link Task} owns the association, because attaching happens from a task in the interface
 * and letting both sides own it is how a join table ends up with orphan rows.
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

    @ManyToMany(mappedBy = "documents")
    private Set<Task> tasks = new LinkedHashSet<>();

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
