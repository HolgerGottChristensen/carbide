pub mod open_dialog;
pub mod save_dialog;
pub mod color_dialog;
pub mod emoji_dialog;
pub mod font_dialog;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FileSpecification {
    pub(crate) name: &'static str,
    pub(crate) extensions: &'static [&'static str],
}

impl FileSpecification {
    pub const fn new(name: &'static str, extensions: &'static [&'static str]) -> Self {
        FileSpecification { name, extensions }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn extensions(&self) -> &[&str] {
        self.extensions
    }
}