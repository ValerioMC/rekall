use axum::Router;

use crate::{backup, catalog, claude, commit, component, connection, context, diagram, filesystem, flow, graph, inventory, note, queue, search, section, settings, skill, step, stream, tag, terminal, timeentry, token, version, vulnerability, wrapup, ApiState};

/// Every `/api` route this module owns.
pub fn router(state: ApiState) -> Router {
    routes()
        .route_layer(axum::middleware::from_fn(rekall_web::extract::uuid_path_params))
        .merge(terminal::socket_routes())
        .merge(connection::credential_routes())
        .merge(backup::routes())
        .merge(settings::routes())
        .with_state(state)
}

pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(catalog::routes())
        .merge(claude::routes())
        .merge(commit::routes())
        .merge(component::routes())
        .merge(connection::routes())
        .merge(context::routes())
        .merge(diagram::routes())
        .merge(filesystem::routes())
        .merge(flow::routes())
        .merge(graph::routes())
        .merge(inventory::routes())
        .merge(note::routes())
        .merge(queue::routes())
        .merge(search::routes())
        .merge(section::routes())
        .merge(skill::routes())
        .merge(step::routes())
        .merge(stream::routes())
        .merge(tag::routes())
        .merge(terminal::routes())
        .merge(timeentry::routes())
        .merge(token::routes())
        .merge(version::routes())
        .merge(vulnerability::routes())
        .merge(wrapup::routes())
}
