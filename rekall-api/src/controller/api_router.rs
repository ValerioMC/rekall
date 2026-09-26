use axum::Router;

use crate::{extract, ApiState};

use super::{
    catalog_controller,
    commit_reference_controller,
    company_controller,
    context_size_controller,
    directory_listing_controller,
    document_controller,
    export_controller,
    search_controller,
    step_event_controller,
    tag_controller,
    task_revision_controller,
    task_step_controller,
    time_entry_controller,
    wrapup_controller,
};

/// Every `/api` route this module owns.
pub fn router(state: ApiState) -> Router {
    routes()
        .route_layer(axum::middleware::from_fn(extract::uuid_path_params))
        .with_state(state)
}

pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(catalog_controller::routes())
        .merge(commit_reference_controller::routes())
        .merge(company_controller::routes())
        .merge(context_size_controller::routes())
        .merge(directory_listing_controller::routes())
        .merge(document_controller::routes())
        .merge(export_controller::routes())
        .merge(search_controller::routes())
        .merge(step_event_controller::routes())
        .merge(tag_controller::routes())
        .merge(task_revision_controller::routes())
        .merge(task_step_controller::routes())
        .merge(time_entry_controller::routes())
        .merge(wrapup_controller::routes())
}
