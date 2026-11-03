use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardMode {
    BeforeTool,
    BeforeStop,
}

impl FromStr for GuardMode {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "before-tool" => Ok(Self::BeforeTool),
            "before-stop" => Ok(Self::BeforeStop),
            other => Err(format!("`{other}` is not a guard event; use before-tool or before-stop")),
        }
    }
}
