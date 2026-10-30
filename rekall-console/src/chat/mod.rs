//! The chat view of a session as the console drives it: following its events, sending it
//! messages, stopping its turn and answering its permission prompts. Opening, listing and closing
//! one stay with the terminals. A terminal's id is refused here.

mod service;

pub use service::chat_service::ChatService;
