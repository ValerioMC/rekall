//! Finds a phrase in the text the console's filter does not reach: task descriptions, steps (title
//! and detail) and wrapups, plus notes so they turn up while browsing tasks. The phrase is matched
//! whole and case-insensitively, as typed; `%` and `_` in it are literal. Under three characters
//! there is no search. Each kind returns at most eight hits, newest first, and within those a hit
//! on a title comes before a hit in the body.

mod model;
mod service;

pub use model::search_hit::SearchHit;
pub use model::search_hit_kind::SearchHitKind;
pub use service::search_service::{TERM_MIN, SearchService, excerpt};
