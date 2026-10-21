//! `044`: the systems outside Rekall a company's projects reach, the projects that use each, and
//! the log of every request a terminal made for one's credential. No column holds a credential:
//! that lives in the OS keychain under `secret_ref`.

use super::{changeset, exec};

changeset!(Connection, "044-connection", |manager| {
    exec(manager, &[
        "CREATE TABLE external_connection (
            id TEXT NOT NULL CONSTRAINT pk_external_connection PRIMARY KEY,
            company_id TEXT NOT NULL,
            label VARCHAR(120) NOT NULL,
            title VARCHAR(200) NOT NULL,
            kind VARCHAR(20) NOT NULL,
            base_url VARCHAR(1000),
            kube_context VARCHAR(253),
            secret_ref VARCHAR(80) NOT NULL,
            secret_set_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_external_connection_company FOREIGN KEY (company_id) REFERENCES company (id) ON DELETE CASCADE,
            CONSTRAINT uq_external_connection_label UNIQUE (company_id, label),
            CONSTRAINT uq_external_connection_secret_ref UNIQUE (secret_ref))",
        "CREATE TABLE project_connection (
            id TEXT NOT NULL CONSTRAINT pk_project_connection PRIMARY KEY,
            project_id TEXT NOT NULL,
            connection_id TEXT NOT NULL,
            target VARCHAR(500),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_project_connection_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
            CONSTRAINT fk_project_connection_connection FOREIGN KEY (connection_id) REFERENCES external_connection (id) ON DELETE CASCADE,
            CONSTRAINT uq_project_connection UNIQUE (project_id, connection_id))",
        "CREATE INDEX ix_project_connection_connection ON project_connection (connection_id)",
        "CREATE TABLE connection_access (
            id TEXT NOT NULL CONSTRAINT pk_connection_access PRIMARY KEY,
            connection_id TEXT NOT NULL,
            task_id TEXT,
            terminal_id VARCHAR(64) NOT NULL,
            accessed_at TEXT NOT NULL,
            granted BOOLEAN NOT NULL,
            reason VARCHAR(500),
            CONSTRAINT fk_connection_access_connection FOREIGN KEY (connection_id) REFERENCES external_connection (id) ON DELETE CASCADE,
            CONSTRAINT fk_connection_access_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE SET NULL)",
        "CREATE INDEX ix_connection_access_connection ON connection_access (connection_id, accessed_at)",
    ])
    .await
});
