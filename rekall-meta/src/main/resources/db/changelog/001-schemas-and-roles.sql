--liquibase formatted sql

--changeset rekall:001-create-schemas
--comment: rekall_meta is Liquibase-owned. rekall_data is created empty and handed to the jOOQ engine.
CREATE SCHEMA IF NOT EXISTS rekall_meta;
CREATE SCHEMA IF NOT EXISTS rekall_data;

--changeset rekall:001-create-reader-role splitStatements:false
--comment: Read-only role used by the MCP server. Claude never authenticates as the app role.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'rekall_reader') THEN
        EXECUTE format('CREATE ROLE rekall_reader LOGIN PASSWORD %L', '${readerPassword}');
    ELSE
        EXECUTE format('ALTER ROLE rekall_reader LOGIN PASSWORD %L', '${readerPassword}');
    END IF;
END
$$;

--changeset rekall:001-reader-schema-grants
--comment: USAGE lets the role see the schemas. SELECT on tables is granted separately.
GRANT USAGE ON SCHEMA rekall_data TO rekall_reader;
GRANT USAGE ON SCHEMA rekall_meta TO rekall_reader;

--changeset rekall:001-reader-default-privileges splitStatements:false
--comment: Without this every table the engine generates later would be invisible to the MCP server.
DO $$
BEGIN
    EXECUTE format(
        'ALTER DEFAULT PRIVILEGES FOR ROLE %I IN SCHEMA rekall_data GRANT SELECT ON TABLES TO rekall_reader',
        current_user);
    EXECUTE format(
        'ALTER DEFAULT PRIVILEGES FOR ROLE %I IN SCHEMA rekall_meta GRANT SELECT ON TABLES TO rekall_reader',
        current_user);
END
$$;
