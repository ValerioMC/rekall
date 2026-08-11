package dev.rekall.meta.validation;

import java.util.Set;
import java.util.regex.Pattern;

/**
 * First of the two layers protecting generated DDL from hostile or broken identifiers.
 *
 * <p>This one runs before anything is persisted to the meta-model. The second layer is
 * rendering: every identifier reaching jOOQ goes through {@code DSL.name(...)} with quoted
 * names always on. Neither layer is sufficient alone, and string concatenation into DDL is
 * never acceptable, including in tests.
 */
public final class IdentifierValidator {

    /** Postgres truncates identifiers at 63 bytes, so anything longer would silently collide. */
    private static final Pattern VALID = Pattern.compile("^[a-z][a-z0-9_]{0,62}$");

    /**
     * Reserved words that cannot be used as a table or column name without quoting. Quoting
     * would work, but an entity called {@code select} makes every hand-written query against
     * the database a trap, so it is rejected outright.
     */
    private static final Set<String> RESERVED = Set.of(
            "all", "analyse", "analyze", "and", "any", "array", "as", "asc", "asymmetric",
            "authorization", "binary", "both", "case", "cast", "check", "collate", "collation",
            "column", "concurrently", "constraint", "create", "cross", "current_catalog",
            "current_date", "current_role", "current_schema", "current_time", "current_timestamp",
            "current_user", "default", "deferrable", "desc", "distinct", "do", "else", "end",
            "except", "false", "fetch", "for", "foreign", "freeze", "from", "full", "grant",
            "group", "having", "ilike", "in", "initially", "inner", "intersect", "into", "is",
            "isnull", "join", "lateral", "leading", "left", "like", "limit", "localtime",
            "localtimestamp", "natural", "not", "notnull", "null", "offset", "on", "only", "or",
            "order", "outer", "overlaps", "placing", "primary", "references", "returning",
            "right", "select", "session_user", "similar", "some", "symmetric", "table",
            "tablesample", "then", "to", "trailing", "true", "union", "unique", "user", "using",
            "variadic", "verbose", "when", "where", "window", "with");

    /**
     * Column names Rekall generates on every table. A user field with one of these names would
     * collide with a system column at CREATE TABLE time.
     */
    private static final Set<String> SYSTEM_COLUMNS = Set.of("id", "created_at", "updated_at");

    private IdentifierValidator() {
    }

    /** @throws InvalidIdentifierException if the name cannot be used as a physical table name */
    public static void validateTableName(String name) {
        validateShape(name, "table");
    }

    /** @throws InvalidIdentifierException if the name cannot be used as a physical column name */
    public static void validateColumnName(String name) {
        validateShape(name, "column");
        if (SYSTEM_COLUMNS.contains(name)) {
            throw new InvalidIdentifierException(
                    "'%s' is a system column generated on every table and cannot be redefined".formatted(name));
        }
    }

    private static void validateShape(String name, String kind) {
        if (name == null || name.isBlank()) {
            throw new InvalidIdentifierException("A %s name is required".formatted(kind));
        }
        if (!VALID.matcher(name).matches()) {
            throw new InvalidIdentifierException(
                    ("'%s' is not a valid %s name. Use lower case letters, digits and underscores, "
                            + "start with a letter, at most 63 characters")
                            .formatted(name, kind));
        }
        if (RESERVED.contains(name)) {
            throw new InvalidIdentifierException(
                    "'%s' is a reserved SQL keyword and cannot be used as a %s name".formatted(name, kind));
        }
    }

    /** Exposed so the UI can warn before the user submits. */
    public static boolean isValidTableName(String name) {
        try {
            validateTableName(name);
            return true;
        } catch (InvalidIdentifierException e) {
            return false;
        }
    }
}
