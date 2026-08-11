package dev.rekall.engine.plan;

/**
 * One reviewable step of a plan.
 *
 * @param sql the statement to execute, or {@code null} when {@link #changeClass} is
 *     {@link ChangeClass#BLOCKED} and there is nothing runnable to show
 * @param inputKey identifies the answer this step is waiting for, {@code null} unless the step
 *     is {@link ChangeClass#NEEDS_INPUT}. Format is {@code table} or {@code table.column}
 */
public record DdlStatement(
        DdlPhase phase, ChangeClass changeClass, String sql, String description, String warning, String inputKey) {

    public static DdlStatement safe(DdlPhase phase, String sql, String description) {
        return new DdlStatement(phase, ChangeClass.SAFE, sql, description, null, null);
    }

    public static DdlStatement needsInput(
            DdlPhase phase, String sql, String description, String warning, String inputKey) {
        return new DdlStatement(phase, ChangeClass.NEEDS_INPUT, sql, description, warning, inputKey);
    }

    public static DdlStatement blocked(DdlPhase phase, String description, String warning) {
        return new DdlStatement(phase, ChangeClass.BLOCKED, null, description, warning, null);
    }

    public boolean isExecutable() {
        return sql != null && changeClass != ChangeClass.BLOCKED;
    }
}
