package dev.rekall.mcp.tool;

import java.util.ArrayList;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

record Anchor(String entityName, String value) {

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

    boolean is(String entity) {
        return entityName != null && entityName.equalsIgnoreCase(entity);
    }

    @Override
    public String toString() {
        return isQualified() ? entityName + ":" + value : value;
    }
}
