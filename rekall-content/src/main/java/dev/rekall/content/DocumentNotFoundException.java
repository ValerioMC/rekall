package dev.rekall.content;

import java.util.UUID;

/** A document id did not match anything. */
public class DocumentNotFoundException extends RuntimeException {

    public DocumentNotFoundException(UUID id) {
        super("No document with id " + id);
    }
}
