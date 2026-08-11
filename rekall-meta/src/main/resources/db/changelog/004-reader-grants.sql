--liquibase formatted sql

--changeset rekall:004-reader-grants-existing
--comment: Default privileges only cover tables created after they were set, so the meta tables
--comment: created above need an explicit grant. Generated tables are covered by the defaults.
GRANT SELECT ON ALL TABLES IN SCHEMA rekall_meta TO rekall_reader;
