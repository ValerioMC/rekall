//! `SettingsController`: the database folders, as the setup screen and the Settings panel manage
//! them. Adding or switching one rewrites `config.json` and restarts onto it.

use std::path::Path;

use axum::extract::{Path as UrlPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use rekall_api::error::{framework, ApiResult};
use rekall_api::extract::JsonBody;
use rekall_common::{Id, Instant, RekallError};
use serde::{Deserialize, Serialize};

use super::folder::inspect;
use super::registry::{DatabaseEntry, DatabaseRegistry, DatabaseRegistryStore};
use crate::restart::Restarter;

#[derive(Clone)]
pub struct SettingsState {
    pub store: DatabaseRegistryStore,
    pub user_home: std::path::PathBuf,
    pub restarter: Restarter,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseView {
    pub id: String,
    pub label: String,
    pub path: String,
    pub active: bool,
    pub reachable: bool,
    pub added_at: String,
    pub last_used_at: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: &'static str,
    pub active: Option<DatabaseView>,
    pub databases: Vec<DatabaseView>,
}

#[derive(Deserialize)]
struct AddRequest {
    path: Option<String>,
    label: Option<String>,
}

#[derive(Serialize)]
struct AddResponse {
    mode: &'static str,
    entry: DatabaseView,
}

#[derive(Deserialize)]
struct RenameRequest {
    label: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckResponse {
    resolved_path: String,
    exists: bool,
    is_directory: bool,
    writable: bool,
    has_database: bool,
    usable: bool,
}

#[derive(Deserialize)]
struct CheckQuery {
    path: Option<String>,
}

pub fn routes(state: SettingsState) -> Router {
    Router::new()
        .route("/api/settings/databases", get(status).post(add))
        .route("/api/settings/databases/check", get(check))
        .route("/api/settings/databases/{id}/activate", post(activate))
        .route("/api/settings/databases/{id}", patch(rename).delete(forget))
        .with_state(state)
}

async fn status(State(state): State<SettingsState>) -> ApiResult<Json<StatusResponse>> {
    let Some(registry) = read(&state)? else {
        return Ok(Json(StatusResponse { status: "SETUP_NEEDED", active: None, databases: Vec::new() }));
    };
    let views: Vec<DatabaseView> = registry.databases.iter().map(|entry| view(&state, entry, &registry)).collect();
    let active_index = views.iter().position(|v| v.active);
    let status = match active_index {
        None => "SETUP_NEEDED",
        Some(index) if views[index].reachable => "READY",
        Some(_) => "UNREACHABLE",
    };
    let active = active_index.map(|index| view(&state, &registry.databases[index], &registry));
    Ok(Json(StatusResponse { status, active, databases: views }))
}

/// `@RequestParam String path`: required, so its absence is Spring's own 400.
async fn check(State(state): State<SettingsState>, Query(query): Query<CheckQuery>) -> Response {
    let Some(path) = query.path else {
        return framework(StatusCode::BAD_REQUEST);
    };
    let result = inspect(Some(&path), &state.user_home);
    let usable = result.usable();
    Json(CheckResponse {
        resolved_path: result.resolved_path,
        exists: result.exists,
        is_directory: result.is_directory,
        writable: result.writable,
        has_database: result.has_database,
        usable,
    })
    .into_response()
}

async fn add(State(state): State<SettingsState>, JsonBody(request): JsonBody<AddRequest>) -> ApiResult<Response> {
    let check = inspect(request.path.as_deref(), &state.user_home);
    if !check.exists || !check.is_directory {
        return Err(RekallError::illegal(format!(
            "The folder \"{}\" does not exist. Create it, then try again.",
            check.resolved_path
        ))
        .into());
    }
    if !check.writable {
        return Err(RekallError::illegal(format!("The folder \"{}\" is not writable.", check.resolved_path)).into());
    }

    let current = read(&state)?.unwrap_or_else(DatabaseRegistry::empty);
    let id = Id::random().to_string();
    let now = Instant::now().to_string();
    let label = match request.label.as_deref().map(str::trim) {
        Some(label) if !label.is_empty() => label.to_string(),
        _ => default_label(&check.resolved_path),
    };
    let entry = DatabaseEntry { id: id.clone(), label, path: check.resolved_path.clone(), added_at: now.clone(), last_used_at: now };

    let mut databases = current.databases.clone();
    databases.push(entry.clone());
    let updated = DatabaseRegistry { active_id: Some(id), databases };
    write(&state, &updated)?;

    state.restarter.restart();
    let mode = if check.has_database { "opened" } else { "created" };
    Ok((StatusCode::CREATED, Json(AddResponse { mode, entry: view(&state, &entry, &updated) })).into_response())
}

async fn activate(State(state): State<SettingsState>, UrlPath(id): UrlPath<String>) -> ApiResult<Json<DatabaseView>> {
    let current = require_registry(&state)?;
    let entry = find(&current, &id)?;
    if !inspect(Some(&entry.path), &state.user_home).is_directory {
        return Err(RekallError::illegal(format!(
            "\"{}\" points at {}, which is not reachable right now.",
            entry.label, entry.path
        ))
        .into());
    }
    let databases = current
        .databases
        .iter()
        .map(|candidate| {
            if candidate.id == id {
                DatabaseEntry { last_used_at: Instant::now().to_string(), ..candidate.clone() }
            } else {
                candidate.clone()
            }
        })
        .collect();
    let registry = DatabaseRegistry { active_id: Some(id.clone()), databases };
    write(&state, &registry)?;

    state.restarter.restart();
    let entry = find(&registry, &id)?;
    Ok(Json(view(&state, entry, &registry)))
}

async fn rename(
    State(state): State<SettingsState>,
    UrlPath(id): UrlPath<String>,
    JsonBody(request): JsonBody<RenameRequest>,
) -> ApiResult<Json<DatabaseView>> {
    let Some(label) = request.label.as_deref().map(str::trim).filter(|l| !l.is_empty()) else {
        return Err(RekallError::illegal("A database needs a name.").into());
    };
    let current = require_registry(&state)?;
    find(&current, &id)?;
    let databases = current
        .databases
        .iter()
        .map(|candidate| {
            if candidate.id == id {
                DatabaseEntry { label: label.to_string(), ..candidate.clone() }
            } else {
                candidate.clone()
            }
        })
        .collect();
    let registry = DatabaseRegistry { active_id: current.active_id.clone(), databases };
    write(&state, &registry)?;
    let entry = find(&registry, &id)?;
    Ok(Json(view(&state, entry, &registry)))
}

async fn forget(State(state): State<SettingsState>, UrlPath(id): UrlPath<String>) -> ApiResult<Response> {
    let current = require_registry(&state)?;
    find(&current, &id)?;
    if current.active_id.as_deref() == Some(id.as_str()) {
        return Err(RekallError::conflict(
            "This is the database currently in use. Switch to another one before forgetting it.",
        )
        .into());
    }
    let remaining = current.databases.iter().filter(|entry| entry.id != id).cloned().collect();
    write(&state, &DatabaseRegistry { active_id: current.active_id.clone(), databases: remaining })?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

fn read(state: &SettingsState) -> ApiResult<Option<DatabaseRegistry>> {
    state.store.read().map_err(|failed| RekallError::internal("UncheckedIOException", failed).into())
}

fn write(state: &SettingsState, registry: &DatabaseRegistry) -> ApiResult<()> {
    state.store.write(registry).map_err(|failed| RekallError::internal("UncheckedIOException", failed).into())
}

fn require_registry(state: &SettingsState) -> ApiResult<DatabaseRegistry> {
    read(state)?.ok_or_else(|| RekallError::not_found_msg("No database is configured yet").into())
}

fn find<'a>(registry: &'a DatabaseRegistry, id: &str) -> ApiResult<&'a DatabaseEntry> {
    registry
        .databases
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| RekallError::not_found_msg(format!("No database registered with id {id}")).into())
}

fn default_label(resolved_path: &str) -> String {
    Path::new(resolved_path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Database".into())
}

fn view(state: &SettingsState, entry: &DatabaseEntry, registry: &DatabaseRegistry) -> DatabaseView {
    DatabaseView {
        id: entry.id.clone(),
        label: entry.label.clone(),
        path: entry.path.clone(),
        active: registry.active_id.as_deref() == Some(entry.id.as_str()),
        reachable: inspect(Some(&entry.path), &state.user_home).is_directory,
        added_at: entry.added_at.clone(),
        last_used_at: entry.last_used_at.clone(),
    }
}
