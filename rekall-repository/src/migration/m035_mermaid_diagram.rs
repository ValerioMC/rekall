//! `035`: a diagram becomes Mermaid sheets with their code links, kept as one JSON document. The
//! Semantic Graph rows before it cannot be drawn any more, so the table is rebuilt empty; a
//! session generates a diagram again on request.

use super::{changeset, exec};

changeset!(MermaidDiagram, "035-mermaid-diagram", |manager| {
    exec(manager, &[
        "DROP TABLE diagram",
        "CREATE TABLE diagram (
            id TEXT NOT NULL CONSTRAINT pk_diagram PRIMARY KEY,
            project_id TEXT NOT NULL,
            task_id TEXT,
            title VARCHAR(200) NOT NULL,
            question VARCHAR(4000) NOT NULL,
            kinds VARCHAR(400) NOT NULL,
            document_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_diagram_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
            CONSTRAINT fk_diagram_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE SET NULL)",
        "CREATE INDEX ix_diagram_project ON diagram (project_id, updated_at)",
    ])
    .await
});
