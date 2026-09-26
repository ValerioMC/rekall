use std::collections::HashMap;

/// Command-line arguments over environment variables, with Spring's relaxed names.
#[derive(Clone, Debug, Default)]
pub struct Properties {
    arguments: HashMap<String, String>,
    environment: HashMap<String, String>,
}

impl Properties {
    pub fn new(args: &[String], environment: HashMap<String, String>) -> Self {
        let mut arguments = HashMap::new();
        for arg in args {
            if let Some(rest) = arg.strip_prefix("--") {
                if let Some((name, value)) = rest.split_once('=') {
                    arguments.insert(name.to_string(), value.to_string());
                }
            }
        }
        Self { arguments, environment }
    }

    /// Properties set in code: what the tests use instead of `@DynamicPropertySource`.
    pub fn of(pairs: &[(&str, &str)]) -> Self {
        Self {
            arguments: pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            environment: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        if let Some(value) = self.arguments.get(name) {
            return Some(value.clone());
        }
        if let Some(value) = self.environment.get(name) {
            return Some(value.clone());
        }
        self.environment.get(&relaxed(name)).cloned()
    }

    pub(super) fn number<T: std::str::FromStr>(&self, name: &str, default: T) -> T {
        self.get(name).and_then(|v| v.trim().parse().ok()).unwrap_or(default)
    }

    pub(super) fn flag(&self, name: &str, default: bool) -> bool {
        match self.get(name).map(|v| v.trim().to_lowercase()) {
            Some(v) if matches!(v.as_str(), "true" | "on" | "yes" | "1") => true,
            Some(v) if matches!(v.as_str(), "false" | "off" | "no" | "0") => false,
            _ => default,
        }
    }
}

/// `rekall.claude.cli-path` -> `REKALL_CLAUDE_CLIPATH`: dots to underscores, dashes dropped.
fn relaxed(name: &str) -> String {
    name.replace('.', "_").replace('-', "").to_uppercase()
}
