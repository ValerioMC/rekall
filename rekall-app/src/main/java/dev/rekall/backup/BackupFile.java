package dev.rekall.backup;

import java.time.Instant;

/** One backup in the database folder's {@code backups/}: a zip H2 wrote with {@code BACKUP TO}. */
public record BackupFile(String name, long sizeBytes, Instant createdAt, Reason reason) {

    /** Why a backup was taken; it is the last part of the file's name. */
    public enum Reason {
        AUTO("auto"),
        MANUAL("manual"),
        BEFORE_RESTORE("before-restore"),
        UPLOADED("uploaded");

        private final String slug;

        Reason(String slug) {
            this.slug = slug;
        }

        public String slug() {
            return slug;
        }

        static Reason ofSlug(String slug) {
            for (Reason reason : values()) {
                if (reason.slug.equals(slug)) {
                    return reason;
                }
            }
            throw new IllegalArgumentException("Not a backup reason: " + slug);
        }
    }
}
