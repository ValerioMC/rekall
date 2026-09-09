package dev.rekall.api.controller;

import dev.rekall.common.ConflictException;
import dev.rekall.common.NotFoundException;
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

    @ExceptionHandler(DataIntegrityViolationException.class)
    public ProblemDetail integrityViolation(DataIntegrityViolationException e) {
        String cause = e.getMostSpecificCause().getMessage();
        return ProblemDetail.forStatusAndDetail(HttpStatus.CONFLICT, explain(cause));
    }

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
