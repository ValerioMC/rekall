use async_trait::async_trait;

use crate::connection::{AtlassianCredential, AtlassianFailure, AtlassianSite, ConfluencePage, ConfluenceSpace};

/// Reads a space of a Confluence site, so the console can fill a project from it.
#[async_trait]
pub trait ConfluenceSpaces: Send + Sync {
    /// The space with exactly this key, or none when the site has no such space.
    async fn space(&self, site: &AtlassianSite, credential: &AtlassianCredential, space_key: &str) -> Result<Option<ConfluenceSpace>, AtlassianFailure>;

    /// The spaces whose key or name holds `query` (every one when it is blank), at most `limit` of them.
    async fn spaces(&self, site: &AtlassianSite, credential: &AtlassianCredential, query: &str, limit: u16) -> Result<Vec<ConfluenceSpace>, AtlassianFailure>;

    /// The current pages of the space, at most `limit` of them, each with its body.
    async fn pages(&self, site: &AtlassianSite, credential: &AtlassianCredential, space_key: &str, limit: u16) -> Result<Vec<ConfluencePage>, AtlassianFailure>;
}
