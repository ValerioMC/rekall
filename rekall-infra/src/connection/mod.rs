//! The credential broker: a terminal on a project that binds connections gets helpers that fetch a
//! credential from Rekall when git or kubectl asks for one, so the credential is never in the
//! terminal's environment, on disk or in the session's context. Every hand-out is logged.

mod credential_broker;
mod credential_session;
mod guard_rails;
mod helper_scripts;
mod session_environment;
mod session_kubeconfig;

pub use credential_broker::CredentialBroker;
pub use session_environment::SessionEnvironment;

use rekall_core::connection::{CredentialPurpose, HandOut, Refusal};
