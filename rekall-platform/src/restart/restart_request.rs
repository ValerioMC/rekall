use super::Hook;

/// What the supervisor is asked to do: close with `before_close` still able to reach the
/// database, then run `after_close` before the next start (to swap its file).
pub struct RestartRequest {
    pub before_close: Hook,
    pub after_close: Hook,
}
