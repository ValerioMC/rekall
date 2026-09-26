//! The logged-in account's Claude usage, fetched from Anthropic's OAuth usage endpoint and shaped
//! for the console meter. The four drawn windows are kept in a fixed order, each with a severity
//! from its percentage. A good read is cached for sixty seconds, a degraded one for ten, so a token
//! that was not readable at launch is retried on the next poll. A failed fetch falls back to the
//! last good one so a blip does not blank the meter, and `refresh` skips the cache when the person
//! asks for a new reading.
//!
//! A 429 is the one answer that is obeyed rather than retried: Anthropic's edge rate limits this
//! client, and every further request while the block stands extends it. The `Retry-After` it
//! carries (five minutes when it carries none) becomes a hold during which neither a poll nor a
//! refresh goes out, and the console is told when it ends.

mod claude_usage_service;
mod claude_usage_view;
mod limit;
mod severity;
mod token_source;
mod usage_controller;
mod usage_query;
mod usage_reader;
mod usage_status;

pub use claude_usage_service::ClaudeUsageService;
pub use claude_usage_view::ClaudeUsageView;
pub use limit::Limit;
pub use severity::Severity;
pub use token_source::TokenSource;
pub use usage_controller::routes;
pub use usage_reader::UsageReader;
pub use usage_status::UsageStatus;
