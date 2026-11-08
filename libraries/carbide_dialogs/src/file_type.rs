
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FileType {
    name: &'static str,
    extension: Vec<&'static str>,
}

impl FileType {
    pub fn new(name: &'static str, extension: Vec<&'static str>) -> FileType {
        FileType {
            name,
            extension,
        }
    }

    pub fn extensions(&self) -> &[&str] {
        &self.extension
    }
}