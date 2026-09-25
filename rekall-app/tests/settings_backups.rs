//! `SettingsControllerTest` and `BackupApiTest` over HTTP, plus what the Java suite could not run
//! in a test context: a restore that restarts the application onto the restored file.

mod support;

use std::io::Read;
use std::sync::LazyLock;

use regex::Regex;
use rekall_app::StartOptions;
use serde_json::json;
use support::{app, app_with, App};

// ---------------------------------------------------------------- SettingsControllerTest

async fn add_and_get_id(app: &App, folder: &std::path::Path) -> String {
    let added = app.post("/api/settings/databases", json!({ "path": folder.to_string_lossy() })).await;
    added.get("entry")["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn with_nothing_registered_status_is_setup_needed() {
    let app = app().await;
    let body = app.get("/api/settings/databases").await;
    assert_eq!(body.str("status"), "SETUP_NEEDED");
    assert!(body.get("databases").as_array().unwrap().is_empty());
}

#[tokio::test]
async fn adding_a_folder_that_does_not_exist_is_rejected_and_nothing_is_created() {
    let app = app().await;
    let missing = tempfile::tempdir().unwrap().path().join("does-not-exist");

    let response = app.post("/api/settings/databases", json!({ "path": missing.to_string_lossy() })).await;

    assert_eq!(response.status, 400);
    assert_contains!(response.detail(), "does not exist");
    assert!(!missing.exists());
}

#[tokio::test]
async fn adding_an_existing_empty_folder_registers_and_activates_it_as_a_new_database() {
    let app = app().await;
    let folder = tempfile::tempdir().unwrap();

    let response = app.post("/api/settings/databases", json!({ "path": folder.path().to_string_lossy() })).await;

    assert_eq!(response.status, 201);
    assert_eq!(response.str("mode"), "created");
    let status = app.get("/api/settings/databases").await;
    assert_eq!(status.get("databases").as_array().unwrap().len(), 1);
    assert_eq!(status.get("active")["active"], json!(true));
}

#[tokio::test]
async fn adding_a_folder_that_already_has_a_database_opens_it_instead_of_creating_one() {
    let app = app().await;
    let folder = tempfile::tempdir().unwrap();
    std::fs::File::create(folder.path().join("rekall.mv.db")).unwrap();

    let response = app.post("/api/settings/databases", json!({ "path": folder.path().to_string_lossy() })).await;

    assert_eq!(response.str("mode"), "opened");
}

#[tokio::test]
async fn the_active_database_cannot_be_forgotten() {
    let app = app().await;
    let folder = tempfile::tempdir().unwrap();
    let id = add_and_get_id(&app, folder.path()).await;

    assert_eq!(app.delete(&format!("/api/settings/databases/{id}")).await.status, 409);
}

#[tokio::test]
async fn renaming_changes_the_label_without_touching_which_database_is_active() {
    let app = app().await;
    let folder = tempfile::tempdir().unwrap();
    let id = add_and_get_id(&app, folder.path()).await;

    let renamed = app.patch(&format!("/api/settings/databases/{id}"), json!({ "label": "Renamed" })).await;

    assert_eq!(renamed.str("label"), "Renamed");
    assert_eq!(renamed.get("active"), &json!(true));
}

#[tokio::test]
async fn a_blank_label_is_rejected() {
    let app = app().await;
    let folder = tempfile::tempdir().unwrap();
    let id = add_and_get_id(&app, folder.path()).await;

    assert_eq!(app.patch(&format!("/api/settings/databases/{id}"), json!({ "label": "  " })).await.status, 400);
}

#[tokio::test]
async fn the_live_check_endpoint_never_registers_anything() {
    let app = app().await;
    let folder = tempfile::tempdir().unwrap();

    let check = app.get(&format!("/api/settings/databases/check?path={}", folder.path().display())).await;

    assert_eq!(check.get("usable"), &json!(true));
    assert!(app.get("/api/settings/databases").await.get("databases").as_array().unwrap().is_empty());
}

// ---------------------------------------------------------------- BackupApiTest

static BACKUP_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^rekall-\d{8}-\d{6}(-\d+)?-manual\.zip$").unwrap());

async fn backup_app() -> App {
    app_with(&[("rekall.backup.keep", "3")], StartOptions { no_restart: true, ..StartOptions::default() }).await
}

fn backups_folder(app: &App) -> std::path::PathBuf {
    app.folder.path().join("db/backups")
}

#[tokio::test]
async fn a_backup_taken_now_is_the_database_zipped_in_the_backups_folder_listed_and_downloadable() {
    let app = backup_app().await;
    let taken = app.post_empty("/api/backups").await;

    let name = taken.str("name");
    assert!(BACKUP_NAME.is_match(&name), "{name}");
    assert!(backups_folder(&app).join(&name).is_file());

    let status = app.get("/api/backups").await;
    assert_eq!(status.get("available"), &json!(true));
    let names: Vec<&str> = status.get("backups").as_array().unwrap().iter().map(|b| b["name"].as_str().unwrap()).collect();
    assert!(names.contains(&name.as_str()));

    let response = app.client.get(app.url(&format!("/api/backups/{name}"))).send().await.unwrap();
    assert_eq!(response.headers()["content-type"], "application/zip");
    assert_eq!(response.headers()["content-disposition"], format!("attachment; filename=\"{name}\"").as_str());
    let zip = response.bytes().await.unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();
    let mut entry = archive.by_index(0).unwrap();
    assert_eq!(entry.name(), "rekall.db");
    let mut head = [0u8; 16];
    entry.read_exact(&mut head).unwrap();
    assert_eq!(&head, b"SQLite format 3\0", "the header the restore checks for");
}

#[tokio::test]
async fn past_the_configured_number_the_oldest_backups_are_dropped() {
    let app = backup_app().await;
    for _ in 0..5 {
        app.post_empty("/api/backups").await;
    }
    assert_eq!(app.get("/api/backups").await.get("backups").as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn a_name_that_is_not_one_the_folder_handed_out_is_refused_so_it_can_never_be_a_path() {
    let app = backup_app().await;
    let traversal = app.get("/api/backups/..%2Frekall.db").await.status;
    assert!(traversal == 400 || traversal == 404, "{traversal}");
    assert_eq!(app.post_empty("/api/backups/rekall-20260101-000000-auto.zip/restore").await.status, 400);
}

#[tokio::test]
async fn an_upload_that_is_not_a_backup_is_refused_and_nothing_is_left_behind() {
    let app = backup_app().await;
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(b"not a zip".to_vec()).file_name("backup.zip"));

    let answer = app.client.post(app.url("/api/backups/restore")).multipart(form).send().await.unwrap();

    assert_eq!(answer.status(), 400, "the file is judged before anything else");
    assert!(!app.folder.path().join("db/rekall.db.restoring").exists());
    let uploaded = std::fs::read_dir(backups_folder(&app))
        .map(|entries| entries.filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().ends_with("-uploaded.zip")).count())
        .unwrap_or(0);
    assert_eq!(uploaded, 0);
}

#[tokio::test]
async fn an_in_memory_database_has_no_backups() {
    let app = app_with(&[("spring.datasource.url", "jdbc:h2:mem:rekall")], StartOptions { no_restart: true, ..StartOptions::default() }).await;
    let status = app.get("/api/backups").await;
    assert_eq!(status.get("available"), &json!(false));
    assert_eq!(app.post_empty("/api/backups").await.status, 409);
}

// ---------------------------------------------------------------- restore, with the restart it needs

#[tokio::test]
async fn a_restore_restarts_the_application_onto_the_backup_and_keeps_what_was_there_as_a_backup_too() {
    let mut app = app_with(&[], StartOptions::default()).await;
    let acme = app.a_company("Before the backup").await;
    let taken = app.post_empty("/api/backups").await.str("name");
    app.a_company("After the backup").await;
    assert_eq!(app.get("/api/companies").await.list().len(), 2);

    let generation = app.running.as_ref().unwrap().generation();
    let started = app.post_empty(&format!("/api/backups/{taken}/restore")).await;
    assert_eq!(started.status, 200, "{}", started.text);
    assert_eq!(started.get("restarting"), &json!(true));
    let safety = started.get("previousState")["name"].as_str().unwrap().to_string();
    assert!(safety.ends_with("-before-restore.zip"), "{safety}");

    app.running.as_mut().unwrap().instance_after(generation).await.expect("the application comes back");

    let companies = app.get("/api/companies").await.list();
    let names: Vec<&str> = companies.iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["Before the backup"], "the restored file is the one in use");
    assert_eq!(companies[0]["id"], json!(acme));
    let listed: Vec<String> =
        app.get("/api/backups").await.get("backups").as_array().unwrap().iter().map(|b| b["name"].as_str().unwrap().to_string()).collect();
    assert!(listed.contains(&safety), "the state before the restore can itself be restored");
    app.stop().await;
}

#[tokio::test]
async fn switching_the_database_in_settings_restarts_onto_the_new_folder() {
    let app = app_with(&[], StartOptions::default()).await;
    app.a_company("On the first database").await;
    // The test database comes from `spring.datasource.url`, which overrides the registry as
    // `REKALL_DB_URL` did; start one on the registry alone to see the switch.
    app.stop().await;

    let mut app = app_with(&[("spring.datasource.url", "")], StartOptions::default()).await;
    assert_eq!(app.get("/api/settings/databases").await.str("status"), "SETUP_NEEDED");
    let folder = tempfile::tempdir().unwrap();

    let generation = app.running.as_ref().unwrap().generation();
    let added = app.post("/api/settings/databases", json!({ "path": folder.path().to_string_lossy(), "label": "Work" })).await;
    assert_eq!(added.status, 201);
    app.running.as_mut().unwrap().instance_after(generation).await.expect("the application comes back");

    assert_eq!(app.get("/api/settings/databases").await.str("status"), "READY");
    app.a_company("On the chosen folder").await;
    assert!(folder.path().join("rekall.db").is_file(), "the database now lives in the folder that was picked");
    app.stop().await;
}
