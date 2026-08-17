package dev.rekall.domain;

import jakarta.persistence.CascadeType;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.FetchType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.OneToMany;
import jakarta.persistence.OrderBy;
import jakarta.persistence.Table;
import jakarta.persistence.UniqueConstraint;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Pattern;
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
 * Something you work on over time, holding tasks.
 *
 * <p>Three fields carry the identity, and they are not interchangeable. {@code label} is what an
 * anchor looks up: {@code project:vega} is a lookup on this column, so it is a slug and never a
 * sentence. {@code title} is what a person calls the project and is free to change without
 * breaking a single anchor anyone has written down. {@code description} is the prose.
 *
 * <p>The label is unique within its company rather than globally, because two companies
 * routinely have a project with the same one. A bare {@code project:website} that matches in two
 * companies is reported as ambiguous rather than guessed at.
 */
@Entity
@Table(
        name = "project",
        uniqueConstraints = @UniqueConstraint(name = "uq_project_company_label", columnNames = {"company_id", "label"}))
@Getter
public class Project {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @NotBlank
    @Size(max = 120)
    @Pattern(regexp = Slug.PATTERN, message = "must be a slug: lowercase letters, digits, '-', '_' or '.'")
    @Column(name = "label", nullable = false, length = 120)
    @Setter
    private String label;

    @NotBlank
    @Size(max = 200)
    @Column(name = "title", nullable = false, length = 200)
    @Setter
    private String title;

    @Enumerated(EnumType.STRING)
    @Column(name = "status", nullable = false, length = 20)
    @Setter
    private ProjectStatus status = ProjectStatus.ACTIVE;

    @Column(name = "description", length = 100_000)
    @Setter
    private String description;

    /**
     * The project's own README: how it is built, how it is organised, the conventions to
     * follow. Unlike {@code description} it is markdown and it is meant to be long, so it
     * carries the same length as a note body rather than the field it sits next to on the form.
     */
    @Column(name = "blueprint_markdown", length = 100_000)
    @Setter
    private String blueprintMarkdown;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(
            name = "company_id",
            nullable = false,
            foreignKey = @jakarta.persistence.ForeignKey(name = "fk_project_company"))
    @Setter
    private Company company;

    @OneToMany(mappedBy = "project", cascade = CascadeType.ALL, orphanRemoval = true)
    @OrderBy("label ASC")
    private List<Task> tasks = new ArrayList<>();


    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected Project() {
        // for JPA
    }

    public Project(String label, String title) {
        this.label = label;
        this.title = title;
    }

    public void addTask(Task task) {
        task.setProject(this);
        tasks.add(task);
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof Project that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "Project[" + label + "]";
    }
}
