//! `052`: an advisory carries its severity and whether it flags malware; an exposure is one package
//! version, so a project holding a package at two versions has one for each, and it remembers the
//! commit its verdict was given at; and the scan's scope (high advisories, malware) is kept.

use super::{changeset, exec};

changeset!(AdvisoryReach, "052-advisory-reach", |manager| {
    exec(manager, &[
        "ALTER TABLE security_advisory ADD COLUMN severity VARCHAR(20) NOT NULL DEFAULT 'CRITICAL'",
        "ALTER TABLE security_advisory ADD COLUMN malware BOOLEAN NOT NULL DEFAULT 0",
        "CREATE TABLE vulnerability_exposure_versioned (
            id TEXT NOT NULL CONSTRAINT pk_vulnerability_exposure PRIMARY KEY,
            project_id TEXT NOT NULL,
            advisory_id TEXT NOT NULL,
            ecosystem VARCHAR(20) NOT NULL,
            package_name VARCHAR(300) NOT NULL,
            package_version VARCHAR(100) NOT NULL,
            verdict VARCHAR(20),
            reasoning TEXT,
            conclusion VARCHAR(1000),
            assessed_at TEXT,
            assessed_commit VARCHAR(64),
            fix_task_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_vulnerability_exposure_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
            CONSTRAINT fk_vulnerability_exposure_advisory FOREIGN KEY (advisory_id) REFERENCES security_advisory (id) ON DELETE CASCADE,
            CONSTRAINT fk_vulnerability_exposure_task FOREIGN KEY (fix_task_id) REFERENCES task (id) ON DELETE SET NULL,
            CONSTRAINT uq_vulnerability_exposure UNIQUE (project_id, advisory_id, ecosystem, package_name, package_version))",
        "INSERT INTO vulnerability_exposure_versioned (id, project_id, advisory_id, ecosystem, package_name, package_version, verdict, reasoning, conclusion, assessed_at, fix_task_id, created_at, updated_at)
            SELECT id, project_id, advisory_id, ecosystem, package_name, package_version, verdict, reasoning, conclusion, assessed_at, fix_task_id, created_at, updated_at
            FROM vulnerability_exposure",
        "DROP INDEX ix_vulnerability_exposure_advisory",
        "DROP TABLE vulnerability_exposure",
        "ALTER TABLE vulnerability_exposure_versioned RENAME TO vulnerability_exposure",
        "CREATE INDEX ix_vulnerability_exposure_advisory ON vulnerability_exposure (advisory_id)",
        "CREATE TABLE scan_scope (
            id TEXT NOT NULL CONSTRAINT pk_scan_scope PRIMARY KEY,
            include_high BOOLEAN NOT NULL DEFAULT 0,
            include_malware BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL)",
    ])
    .await
});
