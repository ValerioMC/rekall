use std::collections::HashMap;
use std::sync::Arc;

use rekall_core::picture::{PicturePath, PictureStore, StoredPicture};
use rekall_core::{in_read, Ctx};
use rekall_kernel::{RekallError, Result};
use rekall_store::repository::picture_text as repo;
use rekall_store::TextTable;
use tracing::{error, info, warn};

use crate::picture::rule::picture_cleanup_rules as rules;
use crate::picture::support::cleanup_board::CleanupBoard;
use crate::picture::{PictureCleanupStatusView, UnusedPictureView};

/// Rows read in one transaction of a search.
const PAGE_ROWS: u32 = 200;
const EVERY_ROW: u32 = u32::MAX;

/// How far a search has read, against the rows counted when it started.
struct Reading {
    rows: u64,
    of: u64,
}

/// What a deletion did with the pictures it was given.
#[derive(Default)]
struct Deletion {
    removed: usize,
    freed_bytes: u64,
    kept: usize,
    not_deleted: usize,
}

/// Finds the pictures no text shows and, when asked, deletes them. One run at a time, in the
/// background.
#[derive(Clone)]
pub struct PictureCleanupService {
    ctx: Ctx,
    store: Arc<dyn PictureStore>,
    board: Arc<CleanupBoard>,
}

impl PictureCleanupService {
    pub fn new(ctx: Ctx, store: Arc<dyn PictureStore>) -> Self {
        let board = Arc::new(CleanupBoard::new(ctx.events.clone()));
        Self { ctx, store, board }
    }

    pub fn status(&self) -> PictureCleanupStatusView {
        self.board.status()
    }

    pub fn start(&self) -> Result<()> {
        self.require_pictures()?;
        self.board.begin_search()?;
        let run = self.clone();
        tokio::spawn(async move { run.search().await });
        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        self.require_pictures()?;
        let listed = self.board.answer_list(PictureCleanupStatusView::Deleting { percentage: 0 })?;
        let run = self.clone();
        tokio::spawn(async move { run.delete_listed(listed).await });
        Ok(())
    }

    pub fn discard(&self) -> Result<()> {
        self.require_pictures()?;
        self.board.answer_list(PictureCleanupStatusView::Idle)?;
        Ok(())
    }

    fn require_pictures(&self) -> Result<()> {
        match self.store.folder() {
            Some(_) => Ok(()),
            None => Err(RekallError::conflict("The open database is not a file on disk, so it keeps no pictures.")),
        }
    }

    async fn search(&self) {
        let ended = match self.find_unused().await {
            Ok(pictures) => {
                info!(unused = pictures.len(), "Searched for unused pictures");
                PictureCleanupStatusView::Ready { pictures }
            }
            Err(failure) => {
                error!(%failure, "The search for unused pictures failed");
                PictureCleanupStatusView::Failed { message: failure.to_string() }
            }
        };
        self.board.show(ended);
    }

    async fn find_unused(&self) -> Result<Vec<UnusedPictureView>> {
        let mut unused = self.settled_pictures().await?;
        let tables = in_read!(&self.ctx, |tx| repo::tables(tx.db()).await)?;
        let mut reading = Reading { rows: 0, of: self.rows_in(&tables).await? };
        for table in &tables {
            self.search_table(table, &mut unused, &mut reading).await?;
        }
        Ok(unused.into_iter().map(|picture| UnusedPictureView { picture: picture.path, size_bytes: picture.size_bytes }).collect())
    }

    /// A transaction per page, so writing goes on during a run.
    async fn search_table(&self, table: &TextTable, unused: &mut Vec<StoredPicture>, reading: &mut Reading) -> Result<()> {
        let mut after_row = None;
        loop {
            let page = in_read!(&self.ctx, |tx| {
                repo::page_holding(tx.db(), table, PicturePath::ADDRESS_PREFIX, after_row, PAGE_ROWS).await
            })?;
            rules::forget_shown(unused, &page.texts);
            reading.rows += page.rows;
            self.board.show(PictureCleanupStatusView::Searching { percentage: rules::percentage(reading.rows, reading.of) });
            if page.rows < u64::from(PAGE_ROWS) {
                return Ok(());
            }
            after_row = page.last_row;
        }
    }

    async fn settled_pictures(&self) -> Result<Vec<StoredPicture>> {
        let now = self.ctx.now();
        let mut pictures = self.store.list().await?;
        pictures.retain(|picture| rules::is_settled(picture, now));
        Ok(pictures)
    }

    async fn rows_in(&self, tables: &[TextTable]) -> Result<u64> {
        in_read!(&self.ctx, |tx| {
            let mut rows = 0;
            for table in tables {
                rows += repo::row_count(tx.db(), table).await?;
            }
            Ok(rows)
        })
    }

    async fn delete_listed(&self, listed: Vec<UnusedPictureView>) {
        let ended = match self.delete_still_unused(&listed).await {
            Ok(Deletion { removed, freed_bytes, kept, not_deleted }) => {
                info!(removed, freed_bytes, kept, not_deleted, "Deleted unused pictures");
                PictureCleanupStatusView::Finished { removed, freed_bytes, kept, not_deleted }
            }
            Err(failure) => {
                error!(%failure, "Deleting unused pictures failed before any file was touched");
                PictureCleanupStatusView::Failed { message: failure.to_string() }
            }
        };
        self.board.show(ended);
    }

    /// Asks the texts first and the files' age second, so a picture written in between shows up as
    /// young. A picture already gone from the disk counts nowhere.
    async fn delete_still_unused(&self, listed: &[UnusedPictureView]) -> Result<Deletion> {
        let texts = self.texts_showing_pictures().await?;
        let mut on_disk: HashMap<String, StoredPicture> =
            self.store.list().await?.into_iter().map(|picture| (picture.path.address(), picture)).collect();
        let now = self.ctx.now();
        let mut deletion = Deletion::default();
        for (handled, unused) in listed.iter().enumerate() {
            match on_disk.remove(&unused.picture.address()) {
                Some(picture) if rules::is_still_unused(&picture, &texts, now) => self.delete_one(&picture, &mut deletion).await,
                Some(_) => deletion.kept += 1,
                None => {}
            }
            let percentage = rules::percentage(handled as u64 + 1, listed.len() as u64);
            self.board.show(PictureCleanupStatusView::Deleting { percentage });
        }
        Ok(deletion)
    }

    async fn delete_one(&self, picture: &StoredPicture, deletion: &mut Deletion) {
        match self.store.delete(&picture.path).await {
            Ok(()) => {
                deletion.removed += 1;
                deletion.freed_bytes += picture.size_bytes;
            }
            Err(failure) => {
                warn!(%failure, picture = picture.path.relative(), "Could not delete an unused picture");
                deletion.not_deleted += 1;
            }
        }
    }

    /// One snapshot of the whole database.
    async fn texts_showing_pictures(&self) -> Result<Vec<String>> {
        in_read!(&self.ctx, |tx| {
            let mut texts = Vec::new();
            for table in repo::tables(tx.db()).await? {
                texts.extend(repo::page_holding(tx.db(), &table, PicturePath::ADDRESS_PREFIX, None, EVERY_ROW).await?.texts);
            }
            Ok(texts)
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/service/picture_cleanup_service_tests.rs"]
mod tests;
