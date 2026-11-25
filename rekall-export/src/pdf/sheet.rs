use super::pdf_metrics::PdfMetrics;
use super::placed::Placed;

/// The pages being filled and the cursor on the current one.
pub(super) struct Sheet {
    pages: Vec<Vec<Placed>>,
    cursor: f32,
}

impl Sheet {
    pub(super) fn new() -> Self {
        Self { pages: vec![Vec::new()], cursor: PdfMetrics::margin() }
    }

    pub(super) fn top() -> f32 {
        PdfMetrics::margin()
    }

    pub(super) fn bottom() -> f32 {
        PdfMetrics::page_height() - PdfMetrics::margin()
    }

    pub(super) fn cursor(&self) -> f32 {
        self.cursor
    }

    pub(super) fn room(&self) -> f32 {
        Self::bottom() - self.cursor
    }

    pub(super) fn at_top(&self) -> bool {
        self.cursor <= Self::top() + 0.5
    }

    pub(super) fn page_index(&self) -> usize {
        self.pages.len() - 1
    }

    pub(super) fn advance(&mut self, height: f32) {
        self.cursor += height;
    }

    pub(super) fn ensure(&mut self, height: f32) {
        if self.room() < height && !self.at_top() {
            self.new_page();
        }
    }

    pub(super) fn new_page(&mut self) {
        self.pages.push(Vec::new());
        self.cursor = Self::top();
    }

    pub(super) fn start_part(&mut self, first: bool) {
        if !first {
            self.new_page();
        }
    }

    pub(super) fn push(&mut self, placed: Placed) {
        let last = self.page_index();
        self.pages[last].push(placed);
    }

    pub(super) fn push_on(&mut self, page: usize, placed: Placed) {
        self.pages[page].push(placed);
    }

    pub(super) fn into_pages(self) -> Vec<Vec<Placed>> {
        self.pages
    }
}
