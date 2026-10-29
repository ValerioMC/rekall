use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UsageStatus {
    Ok,
    Unauthenticated,
    RateLimited,
    Unavailable,
}
