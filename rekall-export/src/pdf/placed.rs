use crate::style::Rgb;

use super::text_style::TextStyle;

/// Something drawn on a page. `top`, `baseline` and `y` are measured down from the page's top edge.
#[derive(Debug, Clone)]
pub(super) enum Placed {
    Text { x: f32, baseline: f32, text: String, style: TextStyle },
    Rect { x: f32, top: f32, width: f32, height: f32, fill: Rgb },
    Stroke { x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Rgb },
    Picture { picture: usize, x: f32, top: f32, width: f32, height: f32 },
    Link { x: f32, top: f32, width: f32, height: f32, url: String },
}
