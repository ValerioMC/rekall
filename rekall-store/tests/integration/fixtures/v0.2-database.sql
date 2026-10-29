-- A database as Rekall v0.2.x wrote it: migrated through 030-note-scope, with one task and its wrapup.
PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS "seaql_migrations" ( "version" varchar NOT NULL PRIMARY KEY, "applied_at" integer NOT NULL );
INSERT INTO seaql_migrations VALUES('001-environment',1791209374);
INSERT INTO seaql_migrations VALUES('001-project',1791209374);
INSERT INTO seaql_migrations VALUES('001-task',1791209374);
INSERT INTO seaql_migrations VALUES('001-document',1791209374);
INSERT INTO seaql_migrations VALUES('002-document-task',1791209374);
INSERT INTO seaql_migrations VALUES('002-carry-task-notes',1791209374);
INSERT INTO seaql_migrations VALUES('002-drop-ownerless-notes',1791209374);
INSERT INTO seaql_migrations VALUES('002-document-loses-its-owners',1791209374);
INSERT INTO seaql_migrations VALUES('002-drop-environment',1791209374);
INSERT INTO seaql_migrations VALUES('003-company',1791209374);
INSERT INTO seaql_migrations VALUES('003-holding-company-for-existing-projects',1791209374);
INSERT INTO seaql_migrations VALUES('003-project-belongs-to-a-company',1791209374);
INSERT INTO seaql_migrations VALUES('003-project-name-unique-per-company',1791209374);
INSERT INTO seaql_migrations VALUES('004-project-label-and-title',1791209374);
INSERT INTO seaql_migrations VALUES('004-task-label-and-title',1791209374);
INSERT INTO seaql_migrations VALUES('005-wrapup',1791209374);
INSERT INTO seaql_migrations VALUES('006-time-entry',1791209374);
INSERT INTO seaql_migrations VALUES('007-project-blueprint',1791209374);
INSERT INTO seaql_migrations VALUES('008-project-repo-folder',1791209374);
INSERT INTO seaql_migrations VALUES('009-task-step',1791209374);
INSERT INTO seaql_migrations VALUES('010-task-step-state',1791209374);
INSERT INTO seaql_migrations VALUES('011-claude-session',1791209374);
INSERT INTO seaql_migrations VALUES('012-claude-session-model',1791209374);
INSERT INTO seaql_migrations VALUES('013-claude-session-effort',1791209374);
INSERT INTO seaql_migrations VALUES('014-task-review-state',1791209374);
INSERT INTO seaql_migrations VALUES('015-task-wrapup-directive',1791209374);
INSERT INTO seaql_migrations VALUES('016-task-step-draft',1791209374);
INSERT INTO seaql_migrations VALUES('017-drop-claude-session',1791209374);
INSERT INTO seaql_migrations VALUES('018-drop-task-wrapup-directive',1791209374);
INSERT INTO seaql_migrations VALUES('019-commit-reference',1791209374);
INSERT INTO seaql_migrations VALUES('020-commit-reference-diff',1791209374);
INSERT INTO seaql_migrations VALUES('021-commit-reference-in-context',1791209374);
INSERT INTO seaql_migrations VALUES('022-project-auto-commit',1791209374);
INSERT INTO seaql_migrations VALUES('023-tag',1791209374);
INSERT INTO seaql_migrations VALUES('024-task-revision',1791209374);
INSERT INTO seaql_migrations VALUES('025-document-context-mode',1791209374);
INSERT INTO seaql_migrations VALUES('026-run-queue',1791209374);
INSERT INTO seaql_migrations VALUES('027-project-icon',1791209374);
INSERT INTO seaql_migrations VALUES('028-diagram',1791209374);
INSERT INTO seaql_migrations VALUES('029-task-step-passes',1791209374);
INSERT INTO seaql_migrations VALUES('030-note-scope',1791209374);
CREATE TABLE document_task (
            document_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            CONSTRAINT pk_document_task PRIMARY KEY (document_id, task_id),
            CONSTRAINT fk_document_task_document FOREIGN KEY (document_id) REFERENCES document (id) ON DELETE CASCADE,
            CONSTRAINT fk_document_task_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE);
CREATE TABLE IF NOT EXISTS "document" (id TEXT NOT NULL CONSTRAINT pk_document PRIMARY KEY,
         title VARCHAR(255) NOT NULL,
         kind VARCHAR(40) NOT NULL,
         body_markdown VARCHAR(100000) NOT NULL,
         source_path VARCHAR(500),
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL, context_mode VARCHAR(20) NOT NULL DEFAULT 'FULL', scope_company_id TEXT CONSTRAINT fk_document_scope_company REFERENCES company (id) ON DELETE CASCADE, scope_project_id TEXT CONSTRAINT fk_document_scope_project REFERENCES project (id) ON DELETE CASCADE);
CREATE TABLE company (
        id TEXT NOT NULL CONSTRAINT pk_company PRIMARY KEY,
        name VARCHAR(120) NOT NULL,
        description VARCHAR(100000),
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        CONSTRAINT uq_company_name UNIQUE (name));
INSERT INTO company VALUES('11111111-1111-1111-1111-111111111111','Acme',NULL,'2026-10-01T09:00:00.000000Z','2026-10-01T09:00:00.000000Z');
CREATE TABLE IF NOT EXISTS "project" (id TEXT NOT NULL CONSTRAINT pk_project PRIMARY KEY,
         label VARCHAR(120) NOT NULL,
         status VARCHAR(20) NOT NULL,
         description VARCHAR(100000),
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL,
         company_id TEXT NOT NULL,
         title VARCHAR(200) NOT NULL, blueprint_markdown VARCHAR(100000), repo_folder VARCHAR(1000), auto_commit BOOLEAN NOT NULL DEFAULT 0, icon VARCHAR(40) NOT NULL DEFAULT 'folder',
         CONSTRAINT fk_project_company FOREIGN KEY (company_id) REFERENCES company (id) ON DELETE CASCADE,
         CONSTRAINT uq_project_company_label UNIQUE (company_id, label));
INSERT INTO project VALUES('22222222-2222-2222-2222-222222222222','vega','ACTIVE',NULL,'2026-10-01T09:00:00.000000Z','2026-10-01T09:00:00.000000Z','11111111-1111-1111-1111-111111111111','Vega',NULL,NULL,0,'folder');
CREATE TABLE IF NOT EXISTS "task" (id TEXT NOT NULL CONSTRAINT pk_task PRIMARY KEY,
         label VARCHAR(160) NOT NULL,
         status VARCHAR(20) NOT NULL,
         description VARCHAR(100000),
         project_id TEXT NOT NULL,
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL,
         title VARCHAR(200) NOT NULL, review_state VARCHAR(20) NOT NULL DEFAULT 'OPEN', claimed_at TEXT, accepted_at TEXT, review_note VARCHAR(2000), tag_id TEXT CONSTRAINT fk_task_tag REFERENCES tag (id) ON DELETE SET NULL,
         CONSTRAINT fk_task_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
         CONSTRAINT uq_task_project_label UNIQUE (project_id, label));
INSERT INTO task VALUES('33333333-3333-3333-3333-333333333333','report','TODO',NULL,'22222222-2222-2222-2222-222222222222','2026-10-01T09:00:00.000000Z','2026-10-01T09:00:00.000000Z','Report','OPEN',NULL,NULL,NULL,NULL);
CREATE TABLE wrapup (
        id TEXT NOT NULL CONSTRAINT pk_wrapup PRIMARY KEY,
        task_id TEXT NOT NULL,
        body_markdown VARCHAR(20000) NOT NULL,
        written_by VARCHAR(20) NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        CONSTRAINT uq_wrapup_task UNIQUE (task_id),
        CONSTRAINT fk_wrapup_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE);
INSERT INTO wrapup VALUES('44444444-4444-4444-4444-444444444444','33333333-3333-3333-3333-333333333333','The report renders.','CLAUDE','2026-10-01T09:00:00.000000Z','2026-10-01T09:00:00.000000Z');
CREATE TABLE time_entry (
        id TEXT NOT NULL CONSTRAINT pk_time_entry PRIMARY KEY,
        task_id TEXT NOT NULL,
        started_at TEXT NOT NULL,
        stopped_at TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        CONSTRAINT fk_time_entry_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE);
CREATE TABLE IF NOT EXISTS "task_step" (id TEXT NOT NULL CONSTRAINT pk_task_step PRIMARY KEY,
         task_id TEXT NOT NULL,
         title VARCHAR(200) NOT NULL,
         body_markdown VARCHAR(20000),
         done_at TEXT,
         position INTEGER NOT NULL,
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL,
         state VARCHAR(20) NOT NULL DEFAULT 'DRAFT',
         running_at TEXT,
         claimed_at TEXT, passes_json TEXT NOT NULL DEFAULT '[]',
         CONSTRAINT fk_task_step_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE);
CREATE TABLE commit_reference (
        id TEXT NOT NULL CONSTRAINT pk_commit_reference PRIMARY KEY,
        task_id TEXT NOT NULL,
        step_id TEXT,
        commit_hash VARCHAR(40) NOT NULL,
        comment VARCHAR(200) NOT NULL,
        created_at TEXT NOT NULL, diff TEXT, in_context BOOLEAN NOT NULL DEFAULT 0,
        CONSTRAINT fk_commit_reference_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
        CONSTRAINT fk_commit_reference_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE CASCADE);
CREATE TABLE tag (
            id TEXT NOT NULL CONSTRAINT pk_tag PRIMARY KEY,
            name VARCHAR(60) NOT NULL CONSTRAINT uq_tag_name UNIQUE,
            icon VARCHAR(40) NOT NULL,
            color VARCHAR(40) NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL);
CREATE TABLE task_revision (
            id TEXT NOT NULL CONSTRAINT pk_task_revision PRIMARY KEY,
            task_id TEXT NOT NULL,
            kind VARCHAR(20) NOT NULL,
            body_markdown VARCHAR(100000) NOT NULL,
            written_by VARCHAR(20),
            written_at TEXT,
            created_at TEXT NOT NULL,
            CONSTRAINT fk_task_revision_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE);
CREATE TABLE run_queue (
            id TEXT NOT NULL CONSTRAINT pk_run_queue PRIMARY KEY,
            state VARCHAR(20) NOT NULL,
            start_at TEXT,
            ceiling_percent INTEGER,
            skip_permissions BOOLEAN NOT NULL DEFAULT 0,
            model VARCHAR(20),
            effort VARCHAR(20),
            hold_until TEXT,
            hold_reason VARCHAR(500),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL);
CREATE TABLE run_queue_item (
            id TEXT NOT NULL CONSTRAINT pk_run_queue_item PRIMARY KEY,
            task_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            state VARCHAR(20) NOT NULL,
            detail VARCHAR(500),
            started_at TEXT,
            finished_at TEXT,
            created_at TEXT NOT NULL,
            CONSTRAINT fk_run_queue_item_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE);
CREATE TABLE diagram (
            id TEXT NOT NULL CONSTRAINT pk_diagram PRIMARY KEY,
            project_id TEXT NOT NULL,
            task_id TEXT,
            title VARCHAR(200) NOT NULL,
            question VARCHAR(4000) NOT NULL,
            graph_json TEXT NOT NULL,
            node_count INTEGER NOT NULL,
            edge_count INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_diagram_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
            CONSTRAINT fk_diagram_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE SET NULL);
CREATE INDEX idx_document_task_task ON document_task (task_id);
CREATE INDEX idx_task_step_task ON task_step (task_id, position);
CREATE INDEX ix_task_revision_task_kind ON task_revision (task_id, kind, created_at);
CREATE INDEX ix_run_queue_item_position ON run_queue_item (position);
CREATE INDEX ix_diagram_project ON diagram (project_id, updated_at);
CREATE INDEX idx_document_scope_company ON document (scope_company_id);
CREATE INDEX idx_document_scope_project ON document (scope_project_id);
COMMIT;
