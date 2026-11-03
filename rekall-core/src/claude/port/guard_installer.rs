use rekall_kernel::Result;

use crate::claude::GuardProtection;

/// The guard hooks in Claude Code's `settings.json`. Both writes keep every entry that is not
/// Rekall's and leave a file that is not valid JSON untouched.
pub trait GuardInstaller: Send + Sync {
    fn status(&self) -> GuardProtection;

    fn install(&self) -> Result<GuardProtection>;

    fn uninstall(&self) -> Result<GuardProtection>;
}
