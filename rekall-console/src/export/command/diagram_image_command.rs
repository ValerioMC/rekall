/// The picture of one mermaid fence, drawn by the console and found again by the fence's source.
#[derive(Clone, Debug)]
pub struct DiagramImageCommand {
    pub source: String,
    pub png: Vec<u8>,
}
