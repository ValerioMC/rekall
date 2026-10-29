//! `049`: a project's deployment graphs, their steps and the routes between them, and the
//! connection a task's step reaches once a graph is attached. A connection deleted leaves the
//! steps that reached it reaching nothing.

use super::{changeset, exec};

changeset!(DeploymentGraph, "049-deployment-graph", |manager| {
    exec(manager, &[
        "CREATE TABLE deployment_graph (
            id TEXT NOT NULL CONSTRAINT pk_deployment_graph PRIMARY KEY,
            project_id TEXT NOT NULL,
            title VARCHAR(200) NOT NULL,
            description TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_deployment_graph_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_deployment_graph_project ON deployment_graph (project_id)",
        "CREATE TABLE graph_step (
            id TEXT NOT NULL CONSTRAINT pk_graph_step PRIMARY KEY,
            graph_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            title VARCHAR(200) NOT NULL,
            body_markdown TEXT,
            connection_id TEXT,
            max_visits INTEGER,
            created_at TEXT NOT NULL,
            CONSTRAINT fk_graph_step_graph FOREIGN KEY (graph_id) REFERENCES deployment_graph (id) ON DELETE CASCADE,
            CONSTRAINT fk_graph_step_connection FOREIGN KEY (connection_id) REFERENCES external_connection (id) ON DELETE SET NULL)",
        "CREATE INDEX ix_graph_step_graph ON graph_step (graph_id, position)",
        "CREATE TABLE graph_route (
            id TEXT NOT NULL CONSTRAINT pk_graph_route PRIMARY KEY,
            graph_id TEXT NOT NULL,
            from_step_id TEXT NOT NULL,
            to_step_id TEXT,
            outcome VARCHAR(60) NOT NULL,
            position INTEGER NOT NULL,
            CONSTRAINT fk_graph_route_graph FOREIGN KEY (graph_id) REFERENCES deployment_graph (id) ON DELETE CASCADE,
            CONSTRAINT fk_graph_route_from FOREIGN KEY (from_step_id) REFERENCES graph_step (id) ON DELETE CASCADE,
            CONSTRAINT fk_graph_route_to FOREIGN KEY (to_step_id) REFERENCES graph_step (id) ON DELETE CASCADE)",
        "CREATE UNIQUE INDEX ux_graph_route_outcome ON graph_route (from_step_id, outcome)",
        "CREATE INDEX ix_graph_route_graph ON graph_route (graph_id)",
        "ALTER TABLE task_step ADD COLUMN connection_id TEXT REFERENCES external_connection (id) ON DELETE SET NULL",
    ])
    .await
});
