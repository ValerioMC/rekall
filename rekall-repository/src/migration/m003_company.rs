//! `003-company.yaml`: projects move under companies. A database that already held projects gets
//! one holding company for them, as the changeset's precondition arranged.

use sea_orm::{ConnectionTrait, Statement};

use super::{changeset, exec, rebuild, NOW};

const HOLDING_COMPANY: &str = "00000000-0000-0000-0000-000000000001";

changeset!(Company, "003-company", |manager| {
    exec(manager, &["CREATE TABLE company (
        id TEXT NOT NULL CONSTRAINT pk_company PRIMARY KEY,
        name VARCHAR(120) NOT NULL,
        description VARCHAR(100000),
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        CONSTRAINT uq_company_name UNIQUE (name))"])
    .await
});

// preConditions onFail: MARK_RAN -- with no project the changeset is recorded and does nothing.
changeset!(HoldingCompanyForExistingProjects, "003-holding-company-for-existing-projects", |manager| {
    let conn = manager.get_connection();
    let any = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT CASE WHEN EXISTS (SELECT 1 FROM project) THEN 1 ELSE 0 END AS present",
        ))
        .await?
        .and_then(|row| row.try_get_by_index::<i64>(0).ok())
        .unwrap_or(0);
    if any != 1 {
        return Ok(());
    }
    exec(manager, &[&format!(
        "INSERT INTO company (id, name, description, created_at, updated_at)
         VALUES ('{HOLDING_COMPANY}', 'Unassigned',
                 'Projects that existed before companies did. Rename or move them.', {NOW}, {NOW})"
    )])
    .await
});

changeset!(ProjectBelongsToACompany, "003-project-belongs-to-a-company", |manager| {
    exec(manager, &[
        "ALTER TABLE project ADD COLUMN company_id TEXT",
        &format!("UPDATE project SET company_id = '{HOLDING_COMPANY}' WHERE company_id IS NULL"),
    ])
    .await?;
    // addNotNullConstraint and addForeignKeyConstraint: a rebuild in SQLite.
    rebuild(
        manager,
        "project",
        "id TEXT NOT NULL CONSTRAINT pk_project PRIMARY KEY,
         name VARCHAR(120) NOT NULL,
         status VARCHAR(20) NOT NULL,
         description VARCHAR(100000),
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL,
         company_id TEXT NOT NULL,
         CONSTRAINT uq_project_name UNIQUE (name),
         CONSTRAINT fk_project_company FOREIGN KEY (company_id) REFERENCES company (id) ON DELETE CASCADE",
        "id, name, status, description, created_at, updated_at, company_id",
        &[],
    )
    .await
});

changeset!(ProjectNameUniquePerCompany, "003-project-name-unique-per-company", |manager| {
    rebuild(
        manager,
        "project",
        "id TEXT NOT NULL CONSTRAINT pk_project PRIMARY KEY,
         name VARCHAR(120) NOT NULL,
         status VARCHAR(20) NOT NULL,
         description VARCHAR(100000),
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL,
         company_id TEXT NOT NULL,
         CONSTRAINT fk_project_company FOREIGN KEY (company_id) REFERENCES company (id) ON DELETE CASCADE,
         CONSTRAINT uq_project_company_name UNIQUE (company_id, name)",
        "id, name, status, description, created_at, updated_at, company_id",
        &[],
    )
    .await
});
