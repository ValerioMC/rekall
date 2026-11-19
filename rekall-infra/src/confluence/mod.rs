//! The Confluence reads the console fills a project from, over Confluence's REST API.

mod http_confluence_spaces;
mod storage_format;

pub use http_confluence_spaces::HttpConfluenceSpaces;

pub(crate) use http_confluence_spaces::rest_root;
