//! An SVG drawn as a PNG, with the fonts installed on this machine.

use std::sync::{Arc, LazyLock};

use rekall_kernel::{InternalFailure, RekallError, Result};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{fontdb, ImageHrefResolver, Options, Tree};

use crate::picture::PrintablePictureView;

/// Drawn at twice its natural size, so it stays sharp on paper.
const SCALE: f32 = 2.0;
const MAX_DRAWN_SIDE: f32 = 4096.0;

static SYSTEM_FONTS: LazyLock<Arc<fontdb::Database>> = LazyLock::new(|| {
    let mut fonts = fontdb::Database::new();
    fonts.load_system_fonts();
    Arc::new(fonts)
});

pub(in crate::picture) fn drawn(bytes: &[u8]) -> Result<PrintablePictureView> {
    let tree = Tree::from_data(bytes, &options()).map_err(|failure| RekallError::illegal(format!("The SVG cannot be drawn: {failure}")))?;
    let natural = tree.size();
    let scale = SCALE.min(MAX_DRAWN_SIDE / natural.width().max(natural.height()));
    let mut canvas = Pixmap::new(whole_pixels(natural.width() * scale), whole_pixels(natural.height() * scale))
        .ok_or_else(|| RekallError::illegal("The SVG has no size it can be drawn at."))?;
    resvg::render(&tree, Transform::from_scale(scale, scale), &mut canvas.as_mut());
    let png = canvas.encode_png().map_err(|failure| RekallError::internal(InternalFailure::Encoding, failure))?;
    Ok(PrintablePictureView { png, natural_width: whole_pixels(natural.width()) })
}

/// Only what the file holds is drawn: a reference to another file on this machine is ignored.
fn options() -> Options<'static> {
    Options {
        fontdb: SYSTEM_FONTS.clone(),
        image_href_resolver: ImageHrefResolver { resolve_string: Box::new(|_, _| None), ..ImageHrefResolver::default() },
        ..Options::default()
    }
}

fn whole_pixels(length: f32) -> u32 {
    (length.round() as u32).max(1)
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/rule/picture_svg_tests.rs"]
mod tests;
