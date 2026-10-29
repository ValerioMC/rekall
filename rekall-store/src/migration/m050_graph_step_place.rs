//! `050`: where a deployment graph's step was dragged on its canvas. Null leaves it to the
//! automatic layout.

use super::{changeset, exec};

changeset!(GraphStepPlace, "050-graph-step-place", |manager| {
    exec(manager, &["ALTER TABLE graph_step ADD COLUMN flow_x INTEGER", "ALTER TABLE graph_step ADD COLUMN flow_y INTEGER"]).await
});
