package dev.rekall.domain;

public final class Slug {

    public static final String PATTERN = "^[a-z0-9]+([._-][a-z0-9]+)*$";

    private Slug() {
    }

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
