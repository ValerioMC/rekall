/// What was carried over, table by table.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub changesets_in_source: usize,
    pub rows: Vec<(String, usize)>,
}

impl ImportReport {
    pub fn total_rows(&self) -> usize {
        self.rows.iter().map(|(_, n)| n).sum()
    }
}
