/// Asks the supervisor to start the application again, onto the database the registry now names.
pub trait ApplicationRestart: Send + Sync {
    /// False where nothing listens for it, as in a test.
    fn restart(&self) -> bool;
}
