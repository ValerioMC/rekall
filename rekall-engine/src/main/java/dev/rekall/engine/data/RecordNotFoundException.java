package dev.rekall.engine.data;

import dev.rekall.meta.domain.MetaTable;

import java.util.UUID;

/** A write targeted a row that does not exist. */
public class RecordNotFoundException extends RuntimeException {

    public RecordNotFoundException(MetaTable entity, UUID id) {
        super("No %s with id %s".formatted(entity.getPhysicalName(), id));
    }
}
