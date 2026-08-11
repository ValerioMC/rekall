package dev.rekall.meta.validation;

/** A physical table or column name was rejected before it could reach generated DDL. */
public class InvalidIdentifierException extends RuntimeException {

    public InvalidIdentifierException(String message) {
        super(message);
    }
}
