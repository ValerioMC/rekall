package dev.rekall.domain.claude;

/**
 * What one {@link ClaudeMessage} row is.
 *
 * <p>{@code USER} a prompt typed in the console; {@code ASSISTANT} a block of Claude's reply;
 * {@code TOOL_USE} a tool call, with its name and a truncated view of its input; {@code
 * TOOL_RESULT} what that call returned; {@code RESULT} the end-of-turn summary, its stats in
 * {@code meta}; {@code SYSTEM} a note from Rekall itself; {@code ERROR} a failure surfaced into
 * the transcript.
 */
public enum ClaudeMessageRole {

    USER,

    ASSISTANT,

    TOOL_USE,

    TOOL_RESULT,

    RESULT,

    SYSTEM,

    ERROR
}
