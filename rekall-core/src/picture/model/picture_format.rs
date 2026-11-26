/// The formats a picture is kept in, each with the one extension its files take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Svg,
}

impl PictureFormat {
    const ALL: [Self; 5] = [Self::Png, Self::Jpeg, Self::Gif, Self::Webp, Self::Svg];

    pub fn of_extension(extension: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|format| format.extension() == extension)
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
            Self::Webp => "webp",
            Self::Svg => "svg",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
            Self::Svg => "image/svg+xml",
        }
    }
}
