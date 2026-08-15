package dev.rekall.domain;

/**
 * The shape a label has to have to survive being typed after {@code /rk}.
 *
 * <p>Anchors are separated by spaces, so a label containing one would split into two terms and
 * resolve to nothing: {@code project:report builder} is the anchor {@code project:code} followed
 * by a bare {@report builder}. Rather than quoting at every call site, the label is narrowed at
 * the one point it is written and the rest of the application can assume it.
 *
 * <p>Normalising rather than rejecting, because what a person types is a title and what the
 * anchor needs is an identifier. {@code "Report Builder"} becomes {@code report-builder}, which
 * is what they meant; only a value with nothing usable left in it is refused.
 */
public final class Slug {

    /** What a normalised label looks like: lowercase runs joined by a single separator. */
    public static final String PATTERN = "^[a-z0-9]+([._-][a-z0-9]+)*$";

    private Slug() {
    }

    /**
     * @throws IllegalArgumentException when nothing usable is left, which the API answers as a
     *     400 rather than letting the database constraint decide
     */
    public static String of(String raw) {
        if (raw == null) {
            throw new IllegalArgumentException("A label is required. It is what `/rk` looks up.");
        }
        String slug = raw.trim()
                .toLowerCase()
                .replaceAll("[^a-z0-9._-]+", "-")
                .replaceAll("[._-]{2,}", "-")
                .replaceAll("^[._-]+|[._-]+$", "");
        if (slug.isEmpty()) {
            throw new IllegalArgumentException(
                    "'%s' leaves no usable label. Use letters, digits, '-', '_' or '.'".formatted(raw));
        }
        return slug;
    }
}
