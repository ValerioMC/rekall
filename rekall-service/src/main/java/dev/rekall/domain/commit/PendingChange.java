package dev.rekall.domain.commit;

/** One line of {@code git status --porcelain}: what the working tree holds that HEAD does not. */
record PendingChange(Kind kind, String path) {

    enum Kind {
        ADDED,
        MODIFIED,
        DELETED,
        RENAMED
    }

    /**
     * Reads a porcelain v1 line ({@code XY path}, or {@code XY old -> new} for a rename). An
     * untracked file counts as added: {@code git add -A} is about to stage it.
     */
    static PendingChange parse(String line) {
        if (line.length() < 4) {
            throw new IllegalArgumentException("Unexpected git status line: '" + line + "'");
        }
        char index = line.charAt(0);
        char worktree = line.charAt(1);
        String path = line.substring(3);
        if (index == 'R' || worktree == 'R') {
            int arrow = path.indexOf(" -> ");
            return new PendingChange(Kind.RENAMED, arrow < 0 ? path : path.substring(arrow + 4));
        }
        if (index == '?' || index == 'A' || worktree == 'A') {
            return new PendingChange(Kind.ADDED, path);
        }
        if (index == 'D' || worktree == 'D') {
            return new PendingChange(Kind.DELETED, path);
        }
        return new PendingChange(Kind.MODIFIED, path);
    }
}
