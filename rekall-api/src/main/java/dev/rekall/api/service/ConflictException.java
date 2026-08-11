package dev.rekall.api.service;

/**
 * The request was well formed but the model does not allow it. Mapped to 409.
 *
 * <p>Messages here are shown directly in the UI, so they say what to do instead, not only what
 * went wrong.
 */
public class ConflictException extends RuntimeException {

    public ConflictException(String message) {
        super(message);
    }
}
