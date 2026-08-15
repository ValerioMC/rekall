package dev.rekall.mcp.tool;

import java.util.ArrayList;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * One term of a context request.
 *
 * <p>Canonical form is {@code entity:value}, for example {@code project:vega}. The positional
 * form carries only the value and leaves {@code entityName} null, which is what makes
 * {@code /rk vega report-builder} work when every term is unambiguous.
 *
 * <p>Parsing is deliberately the whole of the syntax: there is no grammar to extend here. An
 * anchor is a pair of strings, and deciding what those strings mean is the meta-model's job.
 *
 * @param entityName the entity part, or null in the positional form
 * @param value the value to look up against the entity's display field
 */
record Anchor(String entityName, String value) {

    /**
     * A term is a run of non-space characters, optionally ending in a quoted section so that a
     * display value containing spaces survives: {@code env:"kmaster14 / vega-dev"}.
     */
    private static final Pattern TERM = Pattern.compile("\\S*\"[^\"]*\"|\\S+");

    static List<Anchor> parseAll(String raw) {
        List<Anchor> anchors = new ArrayList<>();
        Matcher matcher = TERM.matcher(raw == null ? "" : raw);
        while (matcher.find()) {
            anchors.add(parse(matcher.group()));
        }
        if (anchors.isEmpty()) {
            throw new ToolFailure("No anchor given. Pass something like `project:vega task:report-builder`.");
        }
        return anchors;
    }

    private static Anchor parse(String term) {
        int separator = term.indexOf(':');
        int quote = term.indexOf('"');
        // A colon inside the quoted value is part of the value, not the qualifier.
        boolean qualified = separator >= 0 && (quote < 0 || separator < quote);
        if (!qualified) {
            return new Anchor(null, unquote(term));
        }
        String entityName = term.substring(0, separator).trim();
        String value = unquote(term.substring(separator + 1).trim());
        if (entityName.isEmpty() || value.isEmpty()) {
            throw new ToolFailure(
                    "'%s' is not a valid anchor. Use `entity:value`, or a bare value.".formatted(term));
        }
        return new Anchor(entityName, value);
    }

    private static String unquote(String value) {
        if (value.length() >= 2 && value.startsWith("\"") && value.endsWith("\"")) {
            return value.substring(1, value.length() - 1).trim();
        }
        return value;
    }

    boolean isQualified() {
        return entityName != null;
    }

    /** True when this anchor names the given entity, whatever case it was typed in. */
    boolean is(String entity) {
        return entityName != null && entityName.equalsIgnoreCase(entity);
    }

    @Override
    public String toString() {
        return isQualified() ? entityName + ":" + value : value;
    }
}
