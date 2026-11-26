//! Where the pictures a text shows are on disk, said once at its end.

use crate::context::SessionPictures;
use crate::picture::PicturePath;

use super::text::attribute;

impl SessionPictures {
    /// The text a session is handed, ending with the folder when it shows a picture. Addresses stay
    /// as they are: a session saving the text back has to save them.
    pub fn appended_to(&self, text: String) -> String {
        let block = self.block_for(&text);
        if block.is_empty() {
            return text;
        }
        format!("{}\n\n{block}", text.trim_end_matches('\n'))
    }

    pub fn block_for(&self, text: &str) -> String {
        let prefix = PicturePath::ADDRESS_PREFIX;
        match &self.folder {
            Some(folder) if text.contains(prefix) => format!(
                "<pictures folder=\"{}\">A markdown picture `{prefix}<path>` is the file `<folder>/<path>`. Open it with Read when the \
                 work needs it. In text you save, keep the address, never the file path.</pictures>\n",
                attribute(&folder.display().to_string())
            ),
            _ => String::new(),
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/context/render/pictures_tests.rs"]
mod tests;
