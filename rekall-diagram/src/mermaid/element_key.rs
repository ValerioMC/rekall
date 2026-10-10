/// What a link's key addresses in its sheet: a sequence message by its one-based number, or any
/// element by its Mermaid id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementKey<'a> {
    Message(u32),
    Element(&'a str),
}

impl<'a> ElementKey<'a> {
    pub fn parse(key: &'a str) -> Result<Self, String> {
        let Some(number) = key.strip_prefix('#') else {
            return Ok(ElementKey::Element(key));
        };
        match number.parse::<u32>() {
            Ok(0) => Err("messages are numbered from #1".into()),
            Ok(position) => Ok(ElementKey::Message(position)),
            Err(_) => Err(format!("\"{key}\" is not a message number; write `#` and a number, like `#3`")),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/mermaid/element_key_tests.rs"]
mod tests;
