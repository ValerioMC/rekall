package dev.rekall.claude;

/**
 * What a terminal is opened to do, which decides the {@code /rk} line typed into it first.
 * {@link #WORK} loads the task and works it; {@link #PLAN} has the session propose the task's
 * checklist as drafts and build nothing, so a plan needs no session already running on the task.
 */
public enum TerminalMode {
    WORK,
    PLAN;

    /** The line the session opens on, for a task anchored by {@code anchors}. */
    public String firstLine(String anchors) {
        String load = "/rk " + anchors;
        return this == PLAN ? load + " plan" : load;
    }
}
