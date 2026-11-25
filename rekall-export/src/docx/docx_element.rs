use docx_rs::{Paragraph, Table};

pub(super) enum DocxElement {
    Paragraph(Box<Paragraph>),
    Table(Box<Table>),
}

impl DocxElement {
    pub(super) fn paragraph(paragraph: Paragraph) -> Self {
        Self::Paragraph(Box::new(paragraph))
    }

    pub(super) fn table(table: Table) -> Self {
        Self::Table(Box::new(table))
    }
}
