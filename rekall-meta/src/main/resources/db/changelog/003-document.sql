--liquibase formatted sql

--changeset rekall:003-document
--comment: (entity_name, record_id) is a soft reference into rekall_data, whose tables do not
--comment: exist at migration time. Cleanup on entity drop is therefore explicit in the engine.
CREATE TABLE rekall_meta.document (
    id            uuid         PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_name   varchar(63)  NOT NULL,
    record_id     uuid         NOT NULL,
    title         varchar(255) NOT NULL,
    kind          varchar(40)  NOT NULL DEFAULT 'notes',
    body_markdown text         NOT NULL,
    source_path   text,
    position      integer      NOT NULL DEFAULT 0,
    created_at    timestamptz  NOT NULL DEFAULT now(),
    updated_at    timestamptz  NOT NULL DEFAULT now()
);

CREATE INDEX idx_document_record ON rekall_meta.document (entity_name, record_id);

--changeset rekall:003-document-search-vector
--comment: 'simple' rather than a language configuration: the content mixes Italian and English,
--comment: so committing to one stemmer would degrade recall on the other.
ALTER TABLE rekall_meta.document
    ADD COLUMN search_vector tsvector
        GENERATED ALWAYS AS (to_tsvector('simple', title || ' ' || body_markdown)) STORED;

CREATE INDEX idx_document_search ON rekall_meta.document USING GIN (search_vector);
