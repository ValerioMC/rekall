package dev.rekall.domain.revision;

/** What is about to replace a task's text, which decides whether the text it replaces is kept. */
public enum RevisionTrigger {
    /** The console's editor saving as someone types: kept once per editing window. */
    HAND_EDIT,
    /** A session writing over MCP: always kept. */
    CLAUDE_WRITE,
    /** The text is being removed: always kept. */
    DELETION,
    /** An earlier revision is being written back: always kept, so the restore can itself be undone. */
    RESTORE
}
