#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum FontKind {
    Body,
    Bold,
    Italic,
    BoldItalic,
    Code,
}

impl FontKind {
    pub(super) const ALL: [FontKind; 5] = [FontKind::Body, FontKind::Bold, FontKind::Italic, FontKind::BoldItalic, FontKind::Code];

    pub(super) fn of(bold: bool, italic: bool, code: bool) -> Self {
        match (code, bold, italic) {
            (true, _, _) => FontKind::Code,
            (false, true, true) => FontKind::BoldItalic,
            (false, true, false) => FontKind::Bold,
            (false, false, true) => FontKind::Italic,
            (false, false, false) => FontKind::Body,
        }
    }

    pub(super) fn bytes(self) -> &'static [u8] {
        match self {
            FontKind::Body => include_bytes!("../../assets/fonts/FiraSans-Regular.ttf"),
            FontKind::Bold => include_bytes!("../../assets/fonts/FiraSans-Bold.ttf"),
            FontKind::Italic => include_bytes!("../../assets/fonts/FiraSans-Italic.ttf"),
            FontKind::BoldItalic => include_bytes!("../../assets/fonts/FiraSans-BoldItalic.ttf"),
            FontKind::Code => include_bytes!("../../assets/fonts/FiraCode-Regular.ttf"),
        }
    }
}
