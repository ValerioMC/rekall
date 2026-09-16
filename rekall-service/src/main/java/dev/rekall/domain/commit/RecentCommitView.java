package dev.rekall.domain.commit;

import java.time.Instant;

/** One commit of a project's recent log, as the console lists it when picking a commit by hand. */
public record RecentCommitView(String hash, String subject, Instant committedAt) {

    static RecentCommitView of(GitLogReader.LogEntry entry) {
        return new RecentCommitView(entry.hash(), entry.subject(), entry.committedAt());
    }
}
