--liquibase formatted sql

--changeset rekall:002-meta-table
CREATE TABLE rekall_meta.meta_table (
    id               uuid         PRIMARY KEY DEFAULT gen_random_uuid(),
    physical_name    varchar(63)  NOT NULL,
    label            varchar(120) NOT NULL,
    label_plural     varchar(120) NOT NULL,
    description      text         NOT NULL,
    aliases          text[]       NOT NULL DEFAULT '{}',
    display_field_id uuid,
    status           varchar(20)  NOT NULL DEFAULT 'DRAFT',
    created_at       timestamptz  NOT NULL DEFAULT now(),
    updated_at       timestamptz  NOT NULL DEFAULT now(),
    CONSTRAINT uq_meta_table_physical_name UNIQUE (physical_name),
    CONSTRAINT ck_meta_table_status CHECK (status IN ('DRAFT', 'APPLIED', 'MODIFIED')),
    CONSTRAINT ck_meta_table_description CHECK (length(btrim(description)) > 0)
);

--changeset rekall:002-meta-field
CREATE TABLE rekall_meta.meta_field (
    id            uuid         PRIMARY KEY DEFAULT gen_random_uuid(),
    meta_table_id uuid         NOT NULL,
    column_name   varchar(63)  NOT NULL,
    label         varchar(120) NOT NULL,
    description   text         NOT NULL,
    type          varchar(20)  NOT NULL,
    is_nullable   boolean      NOT NULL DEFAULT true,
    default_value text,
    length        integer,
    precision     integer,
    scale         integer,
    enum_values   text[],
    position      integer      NOT NULL DEFAULT 0,
    CONSTRAINT fk_meta_field_table FOREIGN KEY (meta_table_id)
        REFERENCES rekall_meta.meta_table (id) ON DELETE CASCADE,
    CONSTRAINT uq_meta_field_table_column UNIQUE (meta_table_id, column_name),
    CONSTRAINT ck_meta_field_description CHECK (length(btrim(description)) > 0)
);

CREATE INDEX idx_meta_field_table ON rekall_meta.meta_field (meta_table_id);

--changeset rekall:002-meta-table-display-field-fk
--comment: Circular with fk_meta_field_table, which Postgres allows because the column is nullable.
ALTER TABLE rekall_meta.meta_table
    ADD CONSTRAINT fk_meta_table_display_field FOREIGN KEY (display_field_id)
        REFERENCES rekall_meta.meta_field (id) ON DELETE SET NULL;

--changeset rekall:002-meta-relation
CREATE TABLE rekall_meta.meta_relation (
    id              uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    source_table_id uuid        NOT NULL,
    target_table_id uuid        NOT NULL,
    kind            varchar(20) NOT NULL,
    source_field_id uuid,
    join_table_name varchar(63),
    on_delete       varchar(20) NOT NULL DEFAULT 'RESTRICT',
    description     text        NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT fk_meta_relation_source FOREIGN KEY (source_table_id)
        REFERENCES rekall_meta.meta_table (id) ON DELETE CASCADE,
    CONSTRAINT fk_meta_relation_target FOREIGN KEY (target_table_id)
        REFERENCES rekall_meta.meta_table (id) ON DELETE CASCADE,
    CONSTRAINT fk_meta_relation_field FOREIGN KEY (source_field_id)
        REFERENCES rekall_meta.meta_field (id) ON DELETE CASCADE,
    CONSTRAINT ck_meta_relation_kind CHECK (kind IN ('MANY_TO_ONE', 'MANY_TO_MANY')),
    CONSTRAINT ck_meta_relation_on_delete CHECK (on_delete IN ('RESTRICT', 'CASCADE', 'SET_NULL')),
    CONSTRAINT ck_meta_relation_shape CHECK (
        (kind = 'MANY_TO_ONE'  AND source_field_id IS NOT NULL AND join_table_name IS NULL) OR
        (kind = 'MANY_TO_MANY' AND source_field_id IS NULL     AND join_table_name IS NOT NULL)
    )
);

CREATE INDEX idx_meta_relation_source ON rekall_meta.meta_relation (source_table_id);
CREATE INDEX idx_meta_relation_target ON rekall_meta.meta_relation (target_table_id);

--changeset rekall:002-ddl-log
--comment: Replaces the changelog that rekall_data cannot have. Audit trail and rebuild recipe.
CREATE TABLE rekall_meta.ddl_log (
    id         uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id    uuid        NOT NULL,
    sequence   integer     NOT NULL,
    statement  text        NOT NULL,
    status     varchar(20) NOT NULL,
    error      text,
    applied_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ck_ddl_log_status CHECK (status IN ('APPLIED', 'FAILED', 'ROLLED_BACK'))
);

CREATE INDEX idx_ddl_log_plan ON rekall_meta.ddl_log (plan_id, sequence);
