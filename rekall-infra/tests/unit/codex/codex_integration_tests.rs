#![cfg(unix)]

use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::*;
use crate::cli::EnvironmentSource;

struct TestShell;

#[async_trait]
impl EnvironmentSource for TestShell {
    async fn current(&self) -> HashMap<String, String> {
        HashMap::from([("PATH".into(), "/usr/bin:/bin".into())])
    }
}

fn fixture(home: &Path) -> CodexIntegration {
    let bin = home.join("bin");
    fs::create_dir(&bin).unwrap();
    let binary = bin.join("codex");
    fs::write(&binary, include_str!("../../fixtures/fake_codex.sh")).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    CodexIntegration::new(CodexCli::new(home.to_owned(), Arc::new(TestShell)), "http://localhost:47355/mcp".into())
}

#[tokio::test]
async fn app_server_loads_the_models_with_efforts_and_both_rate_limit_windows() {
    let home = tempfile::tempdir().unwrap();
    let codex = fixture(home.path());
    let models = codex.models().await.unwrap();
    assert_eq!(models[0].model, "test-1");
    assert_eq!(models[0].supported_reasoning_efforts[0].reasoning_effort, "high");
    assert_eq!(models[1].model, "test-2");

    let CodexUsage::Ok { primary: Some(primary), secondary: Some(secondary) } = codex.usage(false).await else {
        panic!("expected both usage windows");
    };
    assert_eq!(primary.used_percent, 27.0);
    assert_eq!(primary.window_duration_mins, Some(300));
    assert_eq!(secondary.used_percent, 52.5);
    assert_eq!(secondary.resets_at, Some(1_999_999_999));
}

#[tokio::test]
async fn connect_installs_the_skill_and_registers_the_streamable_http_url() {
    let home = tempfile::tempdir().unwrap();
    let codex = fixture(home.path());
    assert_eq!(codex.setup().await.unwrap(), CodexSetup::NotConnected);
    assert_eq!(codex.install().await.unwrap(), CodexSetup::Connected);
    assert_eq!(codex.setup().await.unwrap(), CodexSetup::Connected);
    assert!(home.path().join(".codex/skills/rekall/SKILL.md").is_file());
}
