use rekall_kernel::Result;

use crate::claude::Installation;

/// Whether Claude Code knows Rekall's MCP endpoint and `/rk` command, and the install that makes it
/// so. Both run the `claude` binary and block.
pub trait ClaudeCodeSetup: Send + Sync {
    fn status(&self) -> Installation;

    fn install(&self) -> Result<Installation>;

    /// Rewrites only the `/rk` command files; the registration and folder setups stay as they are.
    fn update_commands(&self) -> Result<Installation>;
}
