use super::KeychainSource;

pub(super) struct NoKeychain;

impl KeychainSource for NoKeychain {
    fn secrets(&self) -> Vec<String> {
        Vec::new()
    }
}
