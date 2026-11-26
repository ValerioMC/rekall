use rekall_core::token::TokenCount;
use serde::Deserialize;

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CodexTokenCounts {
    #[serde(alias = "total_tokens")]
    pub total_tokens: u64,
    #[serde(alias = "input_tokens")]
    input_tokens: u64,
    #[serde(default, alias = "cached_input_tokens")]
    cached_input_tokens: u64,
    #[serde(default, alias = "cache_write_input_tokens")]
    cache_write_input_tokens: u64,
    #[serde(alias = "output_tokens")]
    output_tokens: u64,
}

impl CodexTokenCounts {
    pub(crate) fn since(self, previous: Self) -> TokenCount {
        let cache_read = self.cached_input_tokens.saturating_sub(previous.cached_input_tokens);
        let cache_creation = self.cache_write_input_tokens.saturating_sub(previous.cache_write_input_tokens);
        TokenCount {
            input: self.input_tokens.saturating_sub(previous.input_tokens).saturating_sub(cache_read).saturating_sub(cache_creation),
            output: self.output_tokens.saturating_sub(previous.output_tokens),
            cache_creation,
            cache_read,
        }
    }
}
