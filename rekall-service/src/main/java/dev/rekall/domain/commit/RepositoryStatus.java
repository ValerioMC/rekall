package dev.rekall.domain.commit;

/**
 * What a project's folder is, git-wise: whether it is a repository at all, the branch it is on,
 * and the identity git would sign a commit with there. {@code branch} is null on a repository
 * with no commit yet; {@code userName}/{@code userEmail} are null when git has none configured.
 */
public record RepositoryStatus(
        String folder,
        boolean exists,
        boolean repository,
        String branch,
        String userName,
        String userEmail) {

    public static RepositoryStatus unset() {
        return new RepositoryStatus(null, false, false, null, null, null);
    }

    public static RepositoryStatus missing(String folder) {
        return new RepositoryStatus(folder, false, false, null, null, null);
    }

    public static RepositoryStatus notARepository(String folder) {
        return new RepositoryStatus(folder, true, false, null, null, null);
    }

    /** Whether git could commit here as someone: a repository with an email to sign with. */
    public boolean canCommit() {
        return repository && userEmail != null;
    }
}
