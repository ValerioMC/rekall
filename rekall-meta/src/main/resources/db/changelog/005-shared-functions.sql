--liquibase formatted sql

--changeset rekall:005-set-updated-at splitStatements:false
--comment: Shared trigger function attached to every generated table. It lives in rekall_meta
--comment: rather than rekall_data because it is fixed infrastructure, not part of the schema
--comment: the engine owns: keeping it here preserves the rule that Liquibase never authors
--comment: anything inside rekall_data.
CREATE OR REPLACE FUNCTION rekall_meta.set_updated_at() RETURNS trigger AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END
$$ LANGUAGE plpgsql;

--changeset rekall:005-reader-execute-grant
GRANT EXECUTE ON FUNCTION rekall_meta.set_updated_at() TO rekall_reader;
