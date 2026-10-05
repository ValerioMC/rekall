-- The schema as release v0.2.x left it (migration 030). A file at that point is re-pointed to
-- this baseline when it is opened; anything older is refused (see `baseline.rs`).

CREATE TABLE document_task (
    document_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    CONSTRAINT pk_document_task PRIMARY KEY (document_id, task_id),
    CONSTRAINT fk_document_task_document FOREIGN KEY (document_id) REFERENCES document (id) ON DELETE CASCADE,
    CONSTRAINT fk_document_task_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE
);

CREATE TABLE document (
    id TEXT NOT NULL CONSTRAINT pk_document PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    kind VARCHAR(40) NOT NULL,
    body_markdown VARCHAR(100000) NOT NULL,
    source_path VARCHAR(500),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    context_mode VARCHAR(20) NOT NULL DEFAULT 'FULL',
    scope_company_id TEXT CONSTRAINT fk_document_scope_company REFERENCES company (id) ON DELETE CASCADE,
    scope_project_id TEXT CONSTRAINT fk_document_scope_project REFERENCES project (id) ON DELETE CASCADE
);

CREATE TABLE company (
    id TEXT NOT NULL CONSTRAINT pk_company PRIMARY KEY,
    name VARCHAR(120) NOT NULL,
    description VARCHAR(100000),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CONSTRAINT uq_company_name UNIQUE (name)
);

CREATE TABLE project (
    id TEXT NOT NULL CONSTRAINT pk_project PRIMARY KEY,
    label VARCHAR(120) NOT NULL,
    status VARCHAR(20) NOT NULL,
    description VARCHAR(100000),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    company_id TEXT NOT NULL,
    title VARCHAR(200) NOT NULL,
    blueprint_markdown VARCHAR(100000),
    repo_folder VARCHAR(1000),
    auto_commit BOOLEAN NOT NULL DEFAULT 0,
    icon VARCHAR(40) NOT NULL DEFAULT 'folder',
    CONSTRAINT fk_project_company FOREIGN KEY (company_id) REFERENCES company (id) ON DELETE CASCADE,
    CONSTRAINT uq_project_company_label UNIQUE (company_id, label)
);

CREATE TABLE task (
    id TEXT NOT NULL CONSTRAINT pk_task PRIMARY KEY,
    label VARCHAR(160) NOT NULL,
    status VARCHAR(20) NOT NULL,
    description VARCHAR(100000),
    project_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    title VARCHAR(200) NOT NULL,
    review_state VARCHAR(20) NOT NULL DEFAULT 'OPEN',
    claimed_at TEXT,
    accepted_at TEXT,
    review_note VARCHAR(2000),
    tag_id TEXT CONSTRAINT fk_task_tag REFERENCES tag (id) ON DELETE SET NULL,
    CONSTRAINT fk_task_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
    CONSTRAINT uq_task_project_label UNIQUE (project_id, label)
);

CREATE TABLE wrapup (
    id TEXT NOT NULL CONSTRAINT pk_wrapup PRIMARY KEY,
    task_id TEXT NOT NULL,
    body_markdown VARCHAR(20000) NOT NULL,
    written_by VARCHAR(20) NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CONSTRAINT uq_wrapup_task UNIQUE (task_id),
    CONSTRAINT fk_wrapup_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE
);

CREATE TABLE time_entry (
    id TEXT NOT NULL CONSTRAINT pk_time_entry PRIMARY KEY,
    task_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    stopped_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CONSTRAINT fk_time_entry_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE
);

CREATE TABLE task_step (
    id TEXT NOT NULL CONSTRAINT pk_task_step PRIMARY KEY,
    task_id TEXT NOT NULL,
    title VARCHAR(200) NOT NULL,
    body_markdown VARCHAR(20000),
    done_at TEXT,
    position INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    state VARCHAR(20) NOT NULL DEFAULT 'DRAFT',
    running_at TEXT,
    claimed_at TEXT,
    passes_json TEXT NOT NULL DEFAULT '[]',
    CONSTRAINT fk_task_step_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE
);

CREATE TABLE commit_reference (
    id TEXT NOT NULL CONSTRAINT pk_commit_reference PRIMARY KEY,
    task_id TEXT NOT NULL,
    step_id TEXT,
    commit_hash VARCHAR(40) NOT NULL,
    comment VARCHAR(200) NOT NULL,
    created_at TEXT NOT NULL,
    diff TEXT,
    in_context BOOLEAN NOT NULL DEFAULT 0,
    CONSTRAINT fk_commit_reference_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
    CONSTRAINT fk_commit_reference_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE CASCADE
);

CREATE TABLE tag (
    id TEXT NOT NULL CONSTRAINT pk_tag PRIMARY KEY,
    name VARCHAR(60) NOT NULL CONSTRAINT uq_tag_name UNIQUE,
    icon VARCHAR(40) NOT NULL,
    color VARCHAR(40) NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE task_revision (
    id TEXT NOT NULL CONSTRAINT pk_task_revision PRIMARY KEY,
    task_id TEXT NOT NULL,
    kind VARCHAR(20) NOT NULL,
    body_markdown VARCHAR(100000) NOT NULL,
    written_by VARCHAR(20),
    written_at TEXT,
    created_at TEXT NOT NULL,
    CONSTRAINT fk_task_revision_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE
);

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
    updated_at TEXT NOT NULL
);

CREATE TABLE run_queue_item (
    id TEXT NOT NULL CONSTRAINT pk_run_queue_item PRIMARY KEY,
    task_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    state VARCHAR(20) NOT NULL,
    detail VARCHAR(500),
    started_at TEXT,
    finished_at TEXT,
    created_at TEXT NOT NULL,
    CONSTRAINT fk_run_queue_item_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE
);

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
    CONSTRAINT fk_diagram_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE SET NULL
);

CREATE INDEX idx_document_task_task ON document_task (task_id);

CREATE INDEX idx_task_step_task ON task_step (task_id, position);

CREATE INDEX ix_task_revision_task_kind ON task_revision (task_id, kind, created_at);

CREATE INDEX ix_run_queue_item_position ON run_queue_item (position);

CREATE INDEX ix_diagram_project ON diagram (project_id, updated_at);

CREATE INDEX idx_document_scope_company ON document (scope_company_id);

CREATE INDEX idx_document_scope_project ON document (scope_project_id);
