/// The picture of one mermaid fence, found again by the fence's `source`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramImage {
    pub source: String,
    pub png: Vec<u8>,
}
