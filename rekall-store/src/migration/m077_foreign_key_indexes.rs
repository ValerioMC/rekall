//! `077`: every foreign-key column leads an index. SQLite creates none, so deleting a task or a
//! step scanned each child table that references it.

use super::{changeset, exec};

changeset!(ForeignKeyIndexes, "077-foreign-key-indexes", |manager| {
    exec(manager, &[
        "CREATE INDEX ix_connection_access_task_id ON connection_access (task_id)",
        "CREATE INDEX ix_diagram_task_id ON diagram (task_id)",
        "CREATE INDEX ix_done_check_step_id ON done_check (step_id)",
        "CREATE INDEX ix_done_check_result_step_id ON done_check_result (step_id)",
        "CREATE INDEX ix_done_check_result_check_id ON done_check_result (check_id)",
        "CREATE INDEX ix_graph_route_to_step_id ON graph_route (to_step_id)",
        "CREATE INDEX ix_graph_step_connection_id ON graph_step (connection_id)",
        "CREATE INDEX ix_run_queue_item_task_id ON run_queue_item (task_id)",
        "CREATE INDEX ix_step_link_to_step_id ON step_link (to_step_id)",
        "CREATE INDEX ix_step_visit_step_id ON step_visit (step_id)",
        "CREATE INDEX ix_task_tag_id ON task (tag_id)",
        "CREATE INDEX ix_task_step_connection_id ON task_step (connection_id)",
        "CREATE INDEX ix_time_entry_task_id ON time_entry (task_id)",
        "CREATE INDEX ix_token_usage_step_id ON token_usage (step_id)",
        "CREATE INDEX ix_vulnerability_exposure_fix_task_id ON vulnerability_exposure (fix_task_id)",
    ])
    .await
});
