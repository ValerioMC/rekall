/// Where a picture stands across the width it is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PictureAlignment {
    Left,
    Center,
    Right,
}

impl PictureAlignment {
    /// How much of the width a picture leaves free goes on its left.
    pub fn share_before(self) -> f64 {
        match self {
            Self::Left => 0.0,
            Self::Center => 0.5,
            Self::Right => 1.0,
        }
    }
}
