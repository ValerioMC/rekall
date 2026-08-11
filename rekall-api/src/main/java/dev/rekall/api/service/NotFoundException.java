package dev.rekall.api.service;

import java.util.UUID;

/** An id did not match anything. Mapped to 404. */
public class NotFoundException extends RuntimeException {

    public NotFoundException(String what, UUID id) {
        super("No %s with id %s".formatted(what, id));
    }

    public NotFoundException(String message) {
        super(message);
    }
}
