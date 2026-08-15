package dev.rekall.api.controller;

import dev.rekall.api.service.ConflictException;
import dev.rekall.api.service.NotFoundException;
import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import lombok.extern.slf4j.Slf4j;
import org.springframework.dao.DataIntegrityViolationException;
import org.springframework.http.HttpStatus;
import org.springframework.http.ProblemDetail;
import org.springframework.http.converter.HttpMessageNotReadableException;
import org.springframework.web.bind.MethodArgumentNotValidException;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.RestControllerAdvice;

import java.util.stream.Collectors;

/**
 * Turns domain failures into responses the UI can show.
 *
 * <p>Everything the user could have caused becomes a 400 or a 409 carrying the original
 * message, because those messages are written to be read. Only genuinely unexpected failures
 * become a 500, and those are the only ones logged with a stack trace.
 */
@RestControllerAdvice
@Slf4j
public class RestExceptionHandler {

    @ExceptionHandler(NotFoundException.class)
    public ProblemDetail notFound(NotFoundException e) {
        return ProblemDetail.forStatusAndDetail(HttpStatus.NOT_FOUND, e.getMessage());
    }

    @ExceptionHandler(UnknownAnchorException.class)
    public ProblemDetail unknownAnchor(UnknownAnchorException e) {
        return ProblemDetail.forStatusAndDetail(HttpStatus.NOT_FOUND, e.getMessage());
    }

    @ExceptionHandler(ConflictException.class)
    public ProblemDetail conflict(ConflictException e) {
        return ProblemDetail.forStatusAndDetail(HttpStatus.CONFLICT, e.getMessage());
    }

    @ExceptionHandler(AmbiguousAnchorException.class)
    public ProblemDetail ambiguousAnchor(AmbiguousAnchorException e) {
        return ProblemDetail.forStatusAndDetail(HttpStatus.CONFLICT, e.getMessage());
    }

    /**
     * A foreign key doing its job is not a server fault.
     *
     * <p>Refusing to delete a row something else still points at is the whole reason for real
     * tables, so the response says what is holding it rather than surfacing a 500 with a
     * database error string in it.
     */
    @ExceptionHandler(DataIntegrityViolationException.class)
    public ProblemDetail integrityViolation(DataIntegrityViolationException e) {
        String cause = e.getMostSpecificCause().getMessage();
        return ProblemDetail.forStatusAndDetail(HttpStatus.CONFLICT, explain(cause));
    }

    /**
     * A taken label is the constraint a person actually meets, so it gets a sentence of its own.
     *
     * <p>Two projects in one company cannot share {@code project:vega}, because the anchor would
     * then name two records and load neither. The database says "Unique index or primary key
     * violation", which is true and useless.
     */
    private String explain(String cause) {
        if (cause == null) {
            return "This change violates a database constraint.";
        }
        if (cause.contains("still referenced")) {
            return "Something still references this record. Delete or repoint those records first.";
        }
        if (cause.contains("UQ_PROJECT_COMPANY_LABEL")) {
            return "Another project in this company already uses that label. An anchor has to name one record.";
        }
        if (cause.contains("UQ_TASK_PROJECT_LABEL")) {
            return "Another task on this project already uses that label. An anchor has to name one record.";
        }
        if (cause.contains("UQ_COMPANY_NAME")) {
            return "A company with that name already exists.";
        }
        return "This change violates a database constraint: " + cause;
    }

    /**
     * A label with nothing usable in it is a rejected input, not a fault.
     *
     * <p>{@code Slug.of} throws this, and without a handler it would reach the catch-all below
     * and be reported as a 500 for what is a typo.
     */
    @ExceptionHandler(IllegalArgumentException.class)
    public ProblemDetail illegalArgument(IllegalArgumentException e) {
        return ProblemDetail.forStatusAndDetail(HttpStatus.BAD_REQUEST, e.getMessage());
    }

    @ExceptionHandler(MethodArgumentNotValidException.class)
    public ProblemDetail validation(MethodArgumentNotValidException e) {
        String detail = e.getBindingResult().getFieldErrors().stream()
                .map(error -> error.getField() + ": " + error.getDefaultMessage())
                .collect(Collectors.joining("; "));
        return ProblemDetail.forStatusAndDetail(HttpStatus.BAD_REQUEST, detail);
    }

    /**
     * A body that will not parse is a client mistake, not a server fault. Without this the
     * generic handler below turns an unknown enum constant or a malformed UUID into a 500,
     * which reads as "the application is broken" when the request simply was.
     */
    @ExceptionHandler(HttpMessageNotReadableException.class)
    public ProblemDetail unreadableBody(HttpMessageNotReadableException e) {
        return ProblemDetail.forStatusAndDetail(HttpStatus.BAD_REQUEST, e.getMostSpecificCause().getMessage());
    }

    @ExceptionHandler(RuntimeException.class)
    public ProblemDetail unexpected(RuntimeException e) {
        log.error("Unhandled failure", e);
        return ProblemDetail.forStatusAndDetail(
                HttpStatus.INTERNAL_SERVER_ERROR, e.getClass().getSimpleName() + ": " + e.getMessage());
    }
}
