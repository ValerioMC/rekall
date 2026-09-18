package dev.rekall.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.Table;
import jakarta.persistence.UniqueConstraint;
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
 * A configured badge a task can carry: a name, a glowing icon key and a glow colour key, both
 * chosen from the fixed sets the console offers rather than free-form values, so every badge the
 * console draws is one it already knows how to render.
 */
@Entity
@Table(name = "tag", uniqueConstraints = @UniqueConstraint(name = "uq_tag_name", columnNames = "name"))
@Getter
public class Tag {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @NotBlank
    @Size(max = 60)
    @Column(name = "name", nullable = false, length = 60)
    @Setter
    private String name;

    @NotBlank
    @Size(max = 40)
    @Column(name = "icon", nullable = false, length = 40)
    @Setter
    private String icon;

    @NotBlank
    @Size(max = 40)
    @Column(name = "color", nullable = false, length = 40)
    @Setter
    private String color;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at", nullable = false)
    private Instant updatedAt;

    protected Tag() {
    }

    public Tag(String name, String icon, String color) {
        this.name = name;
        this.icon = icon;
        this.color = color;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof Tag that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "Tag[" + name + "]";
    }
}
