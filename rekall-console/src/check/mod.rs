//! The checks a person sets to pass before a claim is accepted. Console-only: `rekall-mcp` cannot
//! name `CheckService`, so a session never writes the command that verifies its own work.

mod command;
mod service;
mod view;

pub use command::check_command::CheckCommand;
pub use command::check_order_command::CheckOrderCommand;
pub use service::check_service::CheckService;
pub use view::check_result_view::CheckResultView;
pub use view::check_view::CheckView;
