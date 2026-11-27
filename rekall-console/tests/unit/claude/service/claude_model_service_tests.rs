use async_trait::async_trait;
use rekall_kernel::RekallError;

use super::*;

struct OneModel;

#[async_trait]
impl ClaudeModelReader for OneModel {
    async fn models(&self) -> Result<Vec<ClaudeModel>> {
        Ok(vec![ClaudeModel {
            value: "claude-example-7[1m]".into(),
            display_name: "Example 7".into(),
            resolved_model: None,
            supports_effort: Some(true),
            supported_effort_levels: Some(vec!["high".into()]),
        }])
    }
}

struct Unavailable;

#[async_trait]
impl ClaudeModelReader for Unavailable {
    async fn models(&self) -> Result<Vec<ClaudeModel>> {
        Err(RekallError::conflict("Claude Code models are unavailable"))
    }
}

#[tokio::test]
async fn the_models_come_from_the_reader() {
    let models = ClaudeModelService::new(Arc::new(OneModel)).models().await.unwrap();

    assert_eq!(models[0].value, "claude-example-7[1m]");
}

#[tokio::test]
async fn the_readers_refusal_reaches_the_caller() {
    let refusal = ClaudeModelService::new(Arc::new(Unavailable)).models().await.unwrap_err();

    assert_eq!(refusal, RekallError::conflict("Claude Code models are unavailable"));
}
