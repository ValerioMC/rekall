//! `--migrate-from-h2` against real H2 files, built by H2's own `RunScript` from two fixtures: a
//! dump of a database the Java build wrote and used (every changeset applied), and one as the
//! first release left it (only the 001 changesets), which the import has to carry through every
//! migration after them. Both need a Java runtime and the H2 jar (`REKALL_H2_JAR` or the local
//! Maven repository); without them the tests say so and pass.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use rekall_app::h2::Importer;
use rekall_app::StartOptions;
use support::app_with;

fn importer() -> Option<Importer> {
    match Importer::locate() {
        Ok(importer) => Some(importer),
        Err(missing) => {
            eprintln!("skipped: {missing}");
            None
        }
    }
}

/// A fresh H2 file at `<folder>/rekall.mv.db`, built from `script`, the way the Java build wrote
/// its files (user `rekall`, in the time zone the timestamps were written in, UTC here).
fn h2_database(importer: &Importer, script: &Path, folder: &Path) -> PathBuf {
    let output = Command::new(&importer.java)
        .env("TZ", "UTC")
        .arg("-Duser.timezone=UTC")
        .arg("-cp")
        .arg(&importer.h2_jar)
        .arg("org.h2.tools.RunScript")
        .args(["-url", &format!("jdbc:h2:file:{}/rekall", folder.display()), "-user", "rekall", "-password", "rekall", "-script"])
        .arg(script)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    folder.join("rekall.mv.db")
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

async fn import(importer: &Importer, h2_file: &Path, target: &Path) -> rekall_app::h2::ImportReport {
    // SAFETY: set before any thread that reads the environment is started by this test.
    std::env::set_var("TZ", "UTC");
    importer.import(h2_file, target).await.expect("the import succeeds")
}

#[tokio::test]
async fn a_database_the_java_build_wrote_arrives_whole_with_its_ids_timestamps_and_orders() {
    let Some(importer) = importer() else { return };
    let folder = tempfile::tempdir().unwrap();
    let h2_file = h2_database(&importer, &fixture("h2-current.sql"), folder.path());
    let target = folder.path().join("rekall.db");

    let report = import(&importer, &h2_file, &target).await;

    assert_eq!(report.changesets_in_source, 37);
    let rows: std::collections::HashMap<_, _> = report.rows.iter().cloned().collect();
    assert_eq!(rows["company"], 6);
    assert_eq!(rows["task"], 21);
    assert_eq!(rows["document_task"], 30);
    assert!(h2_file.is_file(), "the H2 file is left as it was");

    let app = app_with(
        &[("spring.datasource.url", &format!("sqlite:{}", target.display()))],
        StartOptions { no_restart: true, ..StartOptions::default() },
    )
    .await;

    let companies: Vec<String> =
        app.get("/api/companies").await.list().iter().map(|c| c["name"].as_str().unwrap().to_string()).collect();
    assert_eq!(companies, ["Acme", "Co 4e99be", "Co ed0e6f", "Co fd0d3d", "Globex", "Globex Corp"]);

    // What the Java server answered for the same file, read back from it before it was dumped.
    let documents = app.get("/api/documents").await.list();
    let shared = documents.iter().find(|d| d["id"] == "7cf60c0c-8817-44ac-8f47-d14662eaaa31").unwrap();
    let order: Vec<&str> = shared["tasks"].as_array().unwrap().iter().map(|t| t["id"].as_str().unwrap()).collect();
    assert_eq!(
        order,
        [
            "370d6431-960b-4c77-952d-34b376a2950d",
            "40d47d02-4102-45e0-8d20-c63f4537bb78",
            "4a734fa3-0a0e-4139-afc3-faf034a75ee5",
            "be6b254e-55d3-43a4-a6ce-8ee3f4e47e5d",
            "68857795-f358-448d-9c3d-8f11638ca39f",
            "eb4dc1a0-95f3-41d8-bf28-144769f2c8e1",
        ],
        "a note's tasks come out in the order the Java server listed them"
    );

    let wrapups = app.get("/api/wrapups").await.list();
    let wrapup = wrapups.iter().find(|w| w["taskId"] == "2550ba76-9e87-4d75-8ab8-9ea96949d328").unwrap();
    assert_eq!(wrapup["updatedAt"], "2026-09-25T08:35:29.642325Z", "timestamps keep their microseconds");

    let entries = app.get("/api/time-entries").await.list();
    assert_eq!(entries[0]["startedAt"], "2026-01-01T09:00:00Z");
    assert_eq!(entries[0]["stoppedAt"], "2026-01-01T10:30:00Z");

    let hits = app.get("/api/search?q=bastion").await.list();
    assert_eq!(hits[0]["taskId"], "7aee22c5-55d1-47bf-aafa-102f57870d14");
    assert_eq!(hits[0]["where"], "project:vega-platform task:retry-policy");
    app.stop().await;
}

#[tokio::test]
async fn a_database_from_the_first_release_is_carried_through_every_later_migration() {
    let Some(importer) = importer() else { return };
    let folder = tempfile::tempdir().unwrap();
    let h2_file = h2_database(&importer, &fixture("h2-legacy-001.sql"), folder.path());
    let target = folder.path().join("rekall.db");

    let report = import(&importer, &h2_file, &target).await;

    assert_eq!(report.changesets_in_source, 4);
    let app = app_with(
        &[("spring.datasource.url", &format!("sqlite:{}", target.display()))],
        StartOptions { no_restart: true, ..StartOptions::default() },
    )
    .await;

    let companies = app.get("/api/companies").await.list();
    assert_eq!(companies.len(), 1);
    assert_eq!(companies[0]["name"], "Unassigned", "projects older than companies get the holding company");

    let projects = app.get("/api/projects").await.list();
    assert_eq!(projects[0]["label"], "progetto-vega", "the name became a label");
    assert_eq!(projects[0]["title"], "Progetto Vega", "and the title");
    assert_eq!(projects[0]["id"], "22222222-2222-4222-8222-222222222222");

    let tasks = app.get("/api/tasks").await.list();
    assert_eq!(tasks[0]["label"], "report-builder");
    assert_eq!(tasks[0]["title"], "Report Builder");
    assert_eq!(tasks[0]["updatedAt"], "2025-01-04T12:30:00.500Z");

    let notes = app.get("/api/documents").await.list();
    assert_eq!(notes.len(), 1, "the task's note carried onto the task, the project's note dropped with its owner");
    assert_eq!(notes[0]["title"], "CONTEXT.md");
    assert_eq!(notes[0]["tasks"][0]["id"], "33333333-3333-4333-8333-333333333333");

    assert_eq!(
        app.count("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'environment'", vec![]).await,
        0,
        "the environment table went the way the changelog took it"
    );
    app.stop().await;
}

#[tokio::test]
async fn an_existing_target_is_never_overwritten() {
    let Some(importer) = importer() else { return };
    let folder = tempfile::tempdir().unwrap();
    let h2_file = h2_database(&importer, &fixture("h2-legacy-001.sql"), folder.path());
    let target = folder.path().join("rekall.db");
    std::fs::write(&target, "precious").unwrap();

    let refused = importer.import(&h2_file, &target).await.unwrap_err();

    assert!(refused.contains("already exists"), "{refused}");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "precious");
}

#[tokio::test]
async fn a_folder_that_holds_only_the_h2_file_is_imported_the_first_time_it_is_opened() {
    let Some(importer) = importer() else { return };
    let folder = tempfile::tempdir().unwrap();
    h2_database(&importer, &fixture("h2-legacy-001.sql"), folder.path());
    std::env::set_var("TZ", "UTC");

    let app = app_with(
        &[("spring.datasource.url", &format!("sqlite:{}", folder.path().join("rekall.db").display()))],
        StartOptions { no_restart: true, ..StartOptions::default() },
    )
    .await;

    assert_eq!(app.get("/api/tasks").await.list()[0]["label"], "report-builder");
    assert!(folder.path().join("rekall.db").is_file());
    app.stop().await;
}
