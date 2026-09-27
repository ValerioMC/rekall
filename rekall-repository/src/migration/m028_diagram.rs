//! `028`: diagrams, a Semantic Graph per row. The graph is one JSON column because it is read
//! and written whole; the counts beside it let a list show sizes without parsing it.

use super::{changeset, exec};

changeset!(Diagram, "028-diagram", |manager| {
    exec(manager, &[
        "CREATE TABLE diagram (
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
            CONSTRAINT fk_diagram_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE SET NULL)",
        "CREATE INDEX ix_diagram_project ON diagram (project_id, updated_at)",
    ])
    .await
});
