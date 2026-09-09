package dev.rekall.domain.claude;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.FetchType;
import jakarta.persistence.ForeignKey;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.Table;
import lombok.Getter;
import org.hibernate.annotations.CreationTimestamp;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/**
 * One entry in a {@link ClaudeSession}'s transcript: a prompt, a block of a reply, a tool call
 * or its result, or an end-of-turn summary. Written once and never edited.
 */
@Entity
@Table(name = "claude_message")
@Getter
public class ClaudeMessage {

    public static final int MAX_CONTENT = 1_000_000;

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", nullable = false, updatable = false)
    private UUID id;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(
            name = "session_id",
            nullable = false,
            updatable = false,
            foreignKey = @ForeignKey(name = "fk_claude_message_session"))
    private ClaudeSession session;

    @Column(name = "seq", nullable = false)
    private int seq;

    @Enumerated(EnumType.STRING)
    @Column(name = "role", nullable = false, length = 16)
    private ClaudeMessageRole role;

    @Column(name = "content", length = MAX_CONTENT)
    private String content;

    @Column(name = "tool_name", length = 120)
    private String toolName;

    @Column(name = "meta", length = 8_000)
    private String meta;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private Instant createdAt;

    protected ClaudeMessage() {
    }

    public ClaudeMessage(ClaudeSession session, int seq, ClaudeMessageRole role, String content, String toolName, String meta) {
        this.session = session;
        this.seq = seq;
        this.role = role;
        this.content = content;
        this.toolName = toolName;
        this.meta = meta;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        return other instanceof ClaudeMessage that && id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id);
    }

    @Override
    public String toString() {
        return "ClaudeMessage[" + seq + " " + role + "]";
    }
}
