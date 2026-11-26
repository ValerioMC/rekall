use super::PictureAlignment;

/// A markdown picture address, whose `#center` or `#right` ending aligns the picture.
pub(crate) struct PictureAddress<'a> {
    pub(crate) path: &'a str,
    pub(crate) alignment: PictureAlignment,
}

impl<'a> PictureAddress<'a> {
    pub(crate) fn read(address: &'a str) -> Self {
        let (path, ending) = address.split_once('#').unwrap_or((address, ""));
        let alignment = match ending {
            "center" => PictureAlignment::Center,
            "right" => PictureAlignment::Right,
            _ => PictureAlignment::Left,
        };
        Self { path, alignment }
    }

    /// A picture on the web is never handed to a writer: its address is printed instead.
    pub(crate) fn is_web(&self) -> bool {
        self.path.starts_with("http://") || self.path.starts_with("https://")
    }
}
