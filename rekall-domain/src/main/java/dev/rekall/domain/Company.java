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
 * Who the work is for. The outermost record, holding projects.
 *
 * <p>{@code name} is unique and is what an anchor carries: {@code company:acme} is a lookup on
 * this column.
 */
@Entity
@Table(name = "company", uniqueConstraints = @UniqueConstraint(name = "uq_company_name", columnNames = "name"))
@Getter
public class Company {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @NotBlank
    @Size(max = 120)
    @Column(name = "name", nullable = false, length = 120)
    @Setter
    private String name;

    @Column(name = "description", length = 100_000)
    @Setter
    private String description;

    @OneToMany(mappedBy = "company", cascade = CascadeType.ALL, orphanRemoval = true)
    @OrderBy("label ASC")
    private List<Project> projects = new ArrayList<>();

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected Company() {
        // for JPA
    }

    public Company(String name) {
        this.name = name;
    }

    public void addProject(Project project) {
        project.setCompany(this);
        projects.add(project);
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof Company that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "Company[" + name + "]";
    }
}
