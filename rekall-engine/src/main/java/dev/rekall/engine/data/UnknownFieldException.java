package dev.rekall.engine.data;

import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaTable;

import java.util.stream.Collectors;

/**
 * A read or write named a column the entity does not have.
 *
 * <p>Raised rather than ignored on purpose. Silently dropping an unknown key would turn a
 * typo in a filter into an answer that is quietly wrong, which is the worst possible failure
 * mode for something whose job is to be a reliable memory.
 */
public class UnknownFieldException extends RuntimeException {

    public UnknownFieldException(MetaTable entity, String fieldName) {
        super("'%s' has no field '%s'. Available fields: %s"
                .formatted(
                        entity.getPhysicalName(),
                        fieldName,
                        entity.getFields().stream().map(MetaField::getColumnName).collect(Collectors.joining(", "))));
    }
}
