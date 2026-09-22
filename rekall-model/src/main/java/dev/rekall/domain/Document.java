package dev.rekall.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
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

@Entity
@Table(name = "document")
@Getter
public class Document {

    public static final int ANCHOR_ID_LENGTH = 8;

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @NotBlank
    @Size(max = 255)
    @Column(name = "title", nullable = false, length = 255)
    @Setter
    private String title;

    @NotBlank
    @Size(max = 40)
    @Column(name = "kind", nullable = false, length = 40)
    @Setter
    private String kind;

    @Column(name = "body_markdown", nullable = false, length = 100_000)
    @Setter
    private String bodyMarkdown;

    @Column(name = "source_path", length = 500)
    @Setter
    private String sourcePath;

    @Enumerated(EnumType.STRING)
    @Column(name = "context_mode", nullable = false, length = 20)
    @Setter
    private DocumentContextMode contextMode = DocumentContextMode.FULL;

    @ManyToMany(mappedBy = "documents")
    private Set<Task> tasks = new LinkedHashSet<>();

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected Document() {
    }

    public Document(String title, String kind, String bodyMarkdown) {
        this.title = title;
        this.kind = kind;
        this.bodyMarkdown = bodyMarkdown;
    }

    /** The short handle a session loads a reference note by: {@code note:} and the first eight characters of its id. */
    public String anchor() {
        return "note:" + id.toString().substring(0, ANCHOR_ID_LENGTH);
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
