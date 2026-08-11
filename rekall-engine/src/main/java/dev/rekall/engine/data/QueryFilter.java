package dev.rekall.engine.data;

import java.util.List;

/**
 * A read against one entity.
 *
 * <p>Conditions combine with AND. There is no OR and no nesting in v1: every question the
 * application is actually for is a conjunction, and an expression tree would need a grammar,
 * a parser and a UI that none of those questions justify yet.
 */
public record QueryFilter(List<Condition> conditions, List<SortSpec> sort, int limit, int offset) {

    /** Ceiling applied to every read, so no single call can exhaust a context window. */
    public static final int MAX_LIMIT = 500;

    public static final int DEFAULT_LIMIT = 50;

    public QueryFilter {
        conditions = conditions == null ? List.of() : List.copyOf(conditions);
        sort = sort == null ? List.of() : List.copyOf(sort);
        limit = limit <= 0 ? DEFAULT_LIMIT : Math.min(limit, MAX_LIMIT);
        offset = Math.max(offset, 0);
    }

    public static QueryFilter all() {
        return new QueryFilter(List.of(), List.of(), DEFAULT_LIMIT, 0);
    }

    public static QueryFilter where(Condition... conditions) {
        return new QueryFilter(List.of(conditions), List.of(), DEFAULT_LIMIT, 0);
    }

    public record Condition(String field, Operator op, Object value) {

        public static Condition eq(String field, Object value) {
            return new Condition(field, Operator.EQ, value);
        }

        public static Condition isNull(String field) {
            return new Condition(field, Operator.IS_NULL, null);
        }
    }

    public record SortSpec(String field, Direction direction) {

        public static SortSpec asc(String field) {
            return new SortSpec(field, Direction.ASC);
        }

        public static SortSpec desc(String field) {
            return new SortSpec(field, Direction.DESC);
        }

        public enum Direction {
            ASC,
            DESC
        }
    }
}
