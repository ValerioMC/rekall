use rekall_core::claude::Installation;
use rekall_kernel::RekallError;

use super::*;

/// Knows Rekall until asked to install, which it refuses.
struct FakeSetup;

fn installation(status: &'static str) -> Installation {
    Installation {
        status,
        endpoint: "http://127.0.0.1:47355/mcp".into(),
        registered_url: None,
        folder_scoped: Vec::new(),
        command_installed: false,
        cli_path: None,
        manual_command: "claude mcp add rekall".into(),
    }
}

impl ClaudeCodeSetup for FakeSetup {
    fn status(&self) -> Installation {
        installation("MISSING")
    }

    fn install(&self) -> Result<Installation> {
        Err(RekallError::conflict("Claude Code is not installed."))
    }
}

#[tokio::test]
async fn the_status_and_the_installs_refusal_come_from_the_setup() {
    let service = ClaudeInstallService::new(Arc::new(FakeSetup));

    assert_eq!(service.status().await.status, "MISSING");
    assert!(matches!(service.install().await, Err(RekallError::Conflict(_))));
}
