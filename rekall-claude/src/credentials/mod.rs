//! Reads the OAuth access token Claude Code keeps for the logged-in account: the macOS keychain
//! (`Claude Code-credentials`) first, then `~/.claude/.credentials.json`. Never writes or
//! refreshes. Every keychain item is read and the token with the latest `expiresAt` wins; one
//! whose `expiresAt` has passed is reported as absent rather than sent, since Anthropic would only
//! answer 401 and a stream of those is what gets this machine rate limited.

mod claude_credentials;
mod keychain_source;
mod no_keychain;
mod security_command_keychain;

pub use claude_credentials::{KEYCHAIN_SERVICE, ClaudeCredentials};
pub use keychain_source::KeychainSource;
