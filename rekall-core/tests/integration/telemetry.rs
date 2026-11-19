//! The Langfuse setting a person saves, and the variables a launch gets from it.

mod support;

use std::sync::Arc;

use rekall_kernel::Id;
use rekall_core::claude::TerminalLaunch;
use rekall_core::connection::InMemorySecretStore;
use rekall_core::telemetry::{TelemetryChange, TelemetrySettingsService, DEFAULT_HOST, SECRET_KEY_ACCOUNT};
use support::{world, World};

fn change(enabled: bool, secret_key: Option<&str>) -> TelemetryChange {
    TelemetryChange {
        enabled,
        host: Some("http://localhost:3000/".into()),
        public_key: Some("pk-lf-one".into()),
        log_content: false,
        secret_key: secret_key.map(String::from),
    }
}

fn launch() -> TerminalLaunch {
    TerminalLaunch {
        project_id: Id::random(),
        task_id: Some(Id::random()),
        anchors: "task:REPORT-BUILDER".into(),
        working_dir: "/work/vega".into(),
        project_label: "vega".into(),
        task_code: Some("report-builder".into()),
        task_title: Some("Report builder".into()),
        connections: Vec::new(),
    }
}

async fn service() -> (World, TelemetrySettingsService, Arc<InMemorySecretStore>) {
    let world = world().await;
    let secrets = Arc::new(InMemorySecretStore::new());
    let settings = TelemetrySettingsService::new(world.services.ctx.clone(), secrets.clone());
    (world, settings, secrets)
}

#[tokio::test]
async fn nothing_saved_reads_as_off_at_the_local_address() {
    let (_world, settings, _) = service().await;

    let view = settings.read().await.unwrap();

    assert!(!view.enabled);
    assert_eq!(view.host, DEFAULT_HOST);
    assert!(!view.secret_saved);
}

#[tokio::test]
async fn saving_keeps_the_secret_key_out_of_the_database_and_the_view() {
    let (_world, settings, secrets) = service().await;

    let view = settings.change(change(true, Some("sk-lf-two"))).await.unwrap();

    assert!(view.enabled && view.secret_saved);
    assert_eq!(view.host, "http://localhost:3000");
    assert_eq!(view.public_key, "pk-lf-one");
    assert!(!serde_json::to_string(&view).unwrap().contains("sk-lf-two"));
    assert!(secrets.contains(SECRET_KEY_ACCOUNT));
}

#[tokio::test]
async fn saving_without_a_secret_key_keeps_the_stored_one() {
    let (_world, settings, _) = service().await;
    settings.change(change(true, Some("sk-lf-two"))).await.unwrap();

    let view = settings.change(change(false, None)).await.unwrap();

    assert!(!view.enabled);
    assert!(view.secret_saved);
}

#[tokio::test]
async fn a_launch_gets_no_variables_while_tracing_is_off() {
    let (_world, settings, _) = service().await;
    settings.change(change(false, Some("sk-lf-two"))).await.unwrap();

    assert!(settings.launch_variables(&launch()).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_launch_gets_no_variables_while_the_secret_key_is_missing() {
    let (_world, settings, _) = service().await;
    settings.change(change(true, None)).await.unwrap();

    assert!(settings.launch_variables(&launch()).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_launch_gets_the_variables_tagged_with_its_project_and_task() {
    let (_world, settings, _) = service().await;
    settings.change(change(true, Some("sk-lf-two"))).await.unwrap();

    let variables = settings.launch_variables(&launch()).await.unwrap();

    assert_eq!(variables["OTEL_EXPORTER_OTLP_ENDPOINT"], "http://localhost:3000/api/public/otel");
    assert_eq!(variables["OTEL_RESOURCE_ATTRIBUTES"], "rekall.project=vega,rekall.task=report-builder");
}

#[tokio::test]
async fn a_host_that_is_not_an_address_is_refused_and_nothing_is_saved() {
    let (_world, settings, _) = service().await;
    let mut bad = change(true, Some("sk-lf-two"));
    bad.host = Some("localhost".into());

    assert!(settings.change(bad).await.unwrap_err().message().contains("http://"));
    assert!(!settings.read().await.unwrap().secret_saved);
}
