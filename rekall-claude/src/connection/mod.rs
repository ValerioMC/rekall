//! The credential broker: a terminal on a project that binds connections gets helpers that fetch a
//! credential from Rekall when git or kubectl asks for one, so the credential is never in the
//! terminal's environment, on disk or in the session's context. Every hand-out is logged.

mod credential_broker;
mod credential_controller;
mod credential_purpose;
mod credential_session;
mod hand_out;
mod helper_scripts;
mod session_environment;
mod session_kubeconfig;

pub use credential_broker::CredentialBroker;
pub use credential_purpose::CredentialPurpose;
pub use hand_out::{HandOut, Refusal};
pub use session_environment::SessionEnvironment;
pub(crate) use credential_controller::routes;
