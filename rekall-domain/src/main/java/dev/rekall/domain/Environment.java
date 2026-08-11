package dev.rekall.domain;

import jakarta.persistence.CascadeType;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
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
import java.util.UUID;

/**
 * Where a task runs: a cluster, a namespace, the local path to a kubeconfig.
 *
 * <p>The reason this is an entity and not three columns on {@code task}: the same environment
 * serves many tasks, and duplicating its coordinates across them is exactly the drift the
 * folder tree suffered from.
 */
@Entity
@Table(
        name = "environment",
        uniqueConstraints = @UniqueConstraint(name = "uq_environment_label", columnNames = "label"))
@Getter
public class Environment {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    /** Human-readable identity, e.g. {@code kmaster14 / stvv-dev}. Carried by an anchor. */
    @NotBlank
    @Size(max = 160)
    @Column(name = "label", nullable = false, length = 160)
    @Setter
    private String label;

    @Size(max = 120)
    @Column(name = "namespace", length = 120)
    @Setter
    private String namespace;

    @Column(name = "kubeconfig_path", length = 500)
    @Setter
    private String kubeconfigPath;

    @OneToMany(mappedBy = "environment")
    private List<Task> tasks = new ArrayList<>();

    @OneToMany(mappedBy = "environment", cascade = CascadeType.ALL, orphanRemoval = true)
    @OrderBy("position ASC")
    private List<Document> documents = new ArrayList<>();

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected Environment() {
        // for JPA
    }

    public Environment(String label) {
        this.label = label;
    }

    public void addDocument(Document document) {
        document.attachTo(this);
        documents.add(document);
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof Environment that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "Environment[" + label + "]";
    }
}
