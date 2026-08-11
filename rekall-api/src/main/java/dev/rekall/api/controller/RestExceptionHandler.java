package dev.rekall.api.controller;

import dev.rekall.api.service.ConflictException;
import dev.rekall.api.service.NotFoundException;
import dev.rekall.content.DocumentNotFoundException;
import dev.rekall.engine.data.RecordNotFoundException;
import dev.rekall.engine.data.UnknownFieldException;
import dev.rekall.engine.execute.PlanNotApplicableException;
import dev.rekall.engine.type.ValueCoercionException;
import dev.rekall.meta.validation.InvalidIdentifierException;
import lombok.extern.slf4j.Slf4j;
import org.springframework.dao.DataIntegrityViolationException;
import org.springframework.http.HttpStatus;
import org.springframework.http.ProblemDetail;
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

    @ExceptionHandler({DocumentNotFoundException.class, RecordNotFoundException.class})
    public ProblemDetail missingRecord(RuntimeException e) {
        return ProblemDetail.forStatusAndDetail(HttpStatus.NOT_FOUND, e.getMessage());
    }

    @ExceptionHandler(ConflictException.class)
    public ProblemDetail conflict(ConflictException e) {
        return ProblemDetail.forStatusAndDetail(HttpStatus.CONFLICT, e.getMessage());
    }

    @ExceptionHandler({InvalidIdentifierException.class, UnknownFieldException.class, ValueCoercionException.class})
    public ProblemDetail badRequest(RuntimeException e) {
        return ProblemDetail.forStatusAndDetail(HttpStatus.BAD_REQUEST, e.getMessage());
    }

    /**
     * A refused plan is a 409 rather than a 400: the request was fine, the model just does not
     * permit it yet, and the message lists exactly what is missing or blocked.
     */
    @ExceptionHandler(PlanNotApplicableException.class)
    public ProblemDetail planNotApplicable(PlanNotApplicableException e) {
        return ProblemDetail.forStatusAndDetail(HttpStatus.CONFLICT, e.getMessage());
    }

    /**
     * A foreign key doing its job is not a server fault.
     *
     * <p>Refusing to delete a row something else still points at is the whole reason for real
     * tables, so the response says what is holding it rather than surfacing a 500 with a
     * PostgreSQL error string in it.
     */
    @ExceptionHandler(DataIntegrityViolationException.class)
    public ProblemDetail integrityViolation(DataIntegrityViolationException e) {
        String cause = e.getMostSpecificCause().getMessage();
        String detail = cause != null && cause.contains("still referenced")
                ? "Something still references this record. Delete or repoint those records first."
                : "This change violates a database constraint: " + cause;
        return ProblemDetail.forStatusAndDetail(HttpStatus.CONFLICT, detail);
    }

    @ExceptionHandler(MethodArgumentNotValidException.class)
    public ProblemDetail validation(MethodArgumentNotValidException e) {
        String detail = e.getBindingResult().getFieldErrors().stream()
                .map(error -> error.getField() + ": " + error.getDefaultMessage())
                .collect(Collectors.joining("; "));
        return ProblemDetail.forStatusAndDetail(HttpStatus.BAD_REQUEST, detail);
    }

    @ExceptionHandler(RuntimeException.class)
    public ProblemDetail unexpected(RuntimeException e) {
        log.error("Unhandled failure", e);
        return ProblemDetail.forStatusAndDetail(
                HttpStatus.INTERNAL_SERVER_ERROR, e.getClass().getSimpleName() + ": " + e.getMessage());
    }
}
