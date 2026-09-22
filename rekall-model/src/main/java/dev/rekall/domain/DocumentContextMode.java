package dev.rekall.domain;

/** How a note travels in a task's context. */
public enum DocumentContextMode {
    /** In full, under every task it is on. */
    FULL,
    /** As its title, a line of it and an anchor; a session loads the rest only when the work needs it. */
    REFERENCE
}
