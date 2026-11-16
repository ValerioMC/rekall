//! `089`: a connection may be a Confluence site. SQLite widens a CHECK only by rebuilding the table,
//! so `external_connection` is created again with the same columns and keys and its rows copied over.

use super::{changeset, exec};

changeset!(ConfluenceConnection, "089-confluence-connection", |manager| {
    exec(manager, &[
        "CREATE TABLE external_connection_confluence (
            id TEXT NOT NULL CONSTRAINT pk_external_connection PRIMARY KEY,
            label VARCHAR(120) NOT NULL,
            title VARCHAR(200) NOT NULL,
            kind VARCHAR(20) NOT NULL,
            base_url VARCHAR(1000),
            kube_context VARCHAR(253),
            proxy_url VARCHAR(1000),
            root_path VARCHAR(300),
            secret_ref VARCHAR(80) NOT NULL,
            secret_set_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL, accept_self_signed BOOLEAN NOT NULL DEFAULT 0,
            CONSTRAINT uq_external_connection_label UNIQUE (label),
            CONSTRAINT uq_external_connection_secret_ref UNIQUE (secret_ref),
            CONSTRAINT ck_external_connection_kind CHECK (kind IN ('GITLAB', 'GITHUB', 'KUBERNETES', 'JIRA', 'GRAFANA', 'CONFLUENCE')))",
        "INSERT INTO external_connection_confluence (id, label, title, kind, base_url, kube_context, proxy_url, root_path, secret_ref, secret_set_at, created_at, updated_at, accept_self_signed) SELECT id, label, title, kind, base_url, kube_context, proxy_url, root_path, secret_ref, secret_set_at, created_at, updated_at, accept_self_signed FROM external_connection",
        "DROP TABLE external_connection",
        "ALTER TABLE external_connection_confluence RENAME TO external_connection",
    ])
    .await
});
