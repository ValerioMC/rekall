use rekall_core::claude::GuardProtectionState;
use rekall_kernel::RekallError;

use super::*;

struct FakeInstaller;

fn protection(state: GuardProtectionState) -> GuardProtection {
    GuardProtection { state, settings_file: "/home/dev/.claude/settings.json".into(), problem: None }
}

impl GuardInstaller for FakeInstaller {
    fn status(&self) -> GuardProtection {
        protection(GuardProtectionState::NotInstalled)
    }

    fn install(&self) -> Result<GuardProtection> {
        Ok(protection(GuardProtectionState::Installed))
    }

    fn uninstall(&self) -> Result<GuardProtection> {
        Err(RekallError::illegal("settings.json is not valid JSON"))
    }
}

#[tokio::test]
async fn each_call_is_answered_by_the_installer() {
    let service = GuardProtectionService::new(Arc::new(FakeInstaller));

    assert_eq!(service.status().await.state, GuardProtectionState::NotInstalled);
    assert_eq!(service.install().await.unwrap().state, GuardProtectionState::Installed);
    assert!(service.uninstall().await.unwrap_err().is_illegal_argument());
}
