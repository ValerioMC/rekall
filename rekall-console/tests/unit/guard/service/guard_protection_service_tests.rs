use async_trait::async_trait;
use rekall_core::guard::GuardProtectionState;
use rekall_kernel::RekallError;

use super::*;

struct FakeInstaller(SessionProvider);

impl FakeInstaller {
    fn protection(&self, state: GuardProtectionState) -> GuardProtection {
        GuardProtection {
            state,
            settings_file: self.0.label().into(),
            configured: state != GuardProtectionState::NotInstalled,
            problem: None,
        }
    }
}

#[async_trait]
impl GuardInstaller for FakeInstaller {
    async fn status(&self) -> Result<GuardProtection> {
        Ok(self.protection(GuardProtectionState::NotInstalled))
    }

    async fn install(&self) -> Result<GuardProtection> {
        Ok(self.protection(GuardProtectionState::Installed))
    }

    async fn uninstall(&self) -> Result<GuardProtection> {
        Err(RekallError::illegal(format!("Invalid {} settings", self.0.label())))
    }
}

#[tokio::test]
async fn each_call_is_answered_by_the_selected_provider() {
    let service = GuardProtectionService::new(
        Arc::new(FakeInstaller(SessionProvider::Claude)),
        Arc::new(FakeInstaller(SessionProvider::Codex)),
    );

    for provider in [SessionProvider::Claude, SessionProvider::Codex] {
        let status = service.status(provider).await.unwrap();
        assert_eq!(status.state, GuardProtectionState::NotInstalled);
        assert_eq!(status.settings_file, provider.label());
        let installed = service.install(provider).await.unwrap();
        assert_eq!(installed.state, GuardProtectionState::Installed);
        assert_eq!(installed.settings_file, provider.label());
        assert!(service.uninstall(provider).await.unwrap_err().to_string().contains(provider.label()));
    }
}
