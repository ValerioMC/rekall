package dev.rekall.mcp.tool;

/**
 * A tool could not do what was asked, for a reason the caller can act on.
 *
 * <p>Distinct from an unexpected error: the message is written for Claude to read and retry
 * with, so it says what was wrong and what would work instead.
 */
public class ToolFailure extends RuntimeException {

    public ToolFailure(String message) {
        super(message);
    }
}
