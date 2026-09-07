package dev.rekall.api.service;

import java.util.UUID;

public class NotFoundException extends RuntimeException {

    public NotFoundException(String what, UUID id) {
        super("No %s with id %s".formatted(what, id));
    }

    public NotFoundException(String message) {
        super(message);
    }
}
