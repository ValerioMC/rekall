/// Where the keychain half of the read comes from; the real one shells out to `security`.
pub trait KeychainSource: Send + Sync {
    fn secrets(&self) -> Vec<String>;
}
