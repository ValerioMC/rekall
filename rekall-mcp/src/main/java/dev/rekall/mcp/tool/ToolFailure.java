package dev.rekall.mcp.tool;

public class ToolFailure extends RuntimeException {

    public ToolFailure(String message) {
        super(message);
    }
}
