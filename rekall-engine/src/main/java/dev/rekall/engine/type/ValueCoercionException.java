package dev.rekall.engine.type;

/** An inbound value could not be represented as the type its field declares. */
public class ValueCoercionException extends RuntimeException {

    public ValueCoercionException(String message, Throwable cause) {
        super(message, cause);
    }
}
